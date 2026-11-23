//! Text that looks like a credential (format.md §27, R-CONF-7, D40).
//!
//! A dossier is committed to git, where a credential stays in the history, so every write of a
//! Markdown file is checked here. The rules are matched by hand with `str` methods. A line gives
//! at most one hit: the first kind that matches, in the order private key, access token, password.

use crate::atomic;
use crate::error::{Code, Error, Result};
use crate::hash;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

/// The file in the state folder that lists the hashes of the lines a person allowed.
pub const FILE: &str = "credential-ok.json";

/// A line that looks like it holds a credential.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// The 1-based line number.
    pub line: usize,
    /// "private key", "access token" or "password".
    pub kind: &'static str,
    /// The whole line, trimmed. It is never printed, only hashed.
    pub text: String,
}

/// The prefixes of access tokens that must be followed by enough token characters.
const TOKEN_PREFIXES: [&str; 11] = [
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "github_pat_",
    "glpat-",
    "xoxb-",
    "xoxp-",
    "xoxa-",
    "sk-",
];

/// How many token characters (letters, digits, `_` and `-`) follow a token prefix at least.
const TOKEN_BODY_MIN: usize = 20;

/// The names that a password line uses before `=` or `:` (case ignored).
const PASSWORD_NAMES: [&str; 9] = [
    "password", "passwd", "pwd", "secret", "token", "api key", "api-key", "api_key", "apikey",
];

/// The characters that end a word of a line, besides spaces.
const WORD_ENDS: [char; 9] = ['"', '\'', '`', '(', ')', ',', ';', '=', ':'];

/// Every line of `text` that looks like it holds a credential, in file order.
pub fn find(text: &str) -> Vec<Hit> {
    text.lines()
        .enumerate()
        .filter_map(|(index, raw)| {
            let line = raw.trim();
            kind_of(line).map(|kind| Hit {
                line: index + 1,
                kind,
                text: line.to_string(),
            })
        })
        .collect()
}

/// The hits of `new` that are new: their line text is no hit line of `old`, and the hash of
/// their line text is not in `allowed`. `old` is the file before the change (None for a new file).
pub fn added(old: Option<&str>, new: &str, allowed: &[String]) -> Vec<Hit> {
    let before: HashSet<String> = old
        .map(|text| find(text).into_iter().map(|hit| hit.text).collect())
        .unwrap_or_default();
    find(new)
        .into_iter()
        .filter(|hit| {
            !before.contains(&hit.text) && !allowed.contains(&hash::sha256_hex(hit.text.as_bytes()))
        })
        .collect()
}

/// The hashes of the lines a person allowed (SHA-256 hex of the trimmed line). A missing file is
/// an empty list; a file that is not a JSON array of strings is Code::Parse naming the file.
pub fn allowed(state: &Path) -> Result<Vec<String>> {
    let path = state.join(FILE);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::new(Code::Io, format!("{}: {error}", path.display()))),
    };
    serde_json::from_slice::<Vec<String>>(&bytes).map_err(|error| {
        Error::new(
            Code::Parse,
            format!("{}: not a JSON array of strings ({error})", path.display()),
        )
    })
}

/// Adds the hashes of these hits to the allowed file (each once, sorted), written with
/// `atomic::replace`. Nothing is written when `hits` is empty.
pub fn allow(state: &Path, hits: &[Hit]) -> Result<()> {
    if hits.is_empty() {
        return Ok(());
    }
    let mut hashes: BTreeSet<String> = allowed(state)?.into_iter().collect();
    hashes.extend(hits.iter().map(|hit| hash::sha256_hex(hit.text.as_bytes())));
    let list: Vec<String> = hashes.into_iter().collect();
    let mut bytes = serde_json::to_string_pretty(&list)
        .map_err(|error| Error::new(Code::Io, error.to_string()))?
        .into_bytes();
    bytes.push(b'\n');
    atomic::replace(&state.join(FILE), &bytes, None)
}

/// "<file>: line 12 (password); line 14 (private key)" for a refusal. Never the line text.
pub fn listed(file: &str, hits: &[Hit]) -> String {
    if hits.is_empty() {
        return file.to_string();
    }
    let parts: Vec<String> = hits
        .iter()
        .map(|hit| format!("line {} ({})", hit.line, hit.kind))
        .collect();
    format!("{file}: {}", parts.join("; "))
}

/// The kind of credential a trimmed line looks like, if any.
fn kind_of(line: &str) -> Option<&'static str> {
    if has_private_key(line) {
        Some("private key")
    } else if has_access_token(line) {
        Some("access token")
    } else if has_password(line) {
        Some("password")
    } else {
        None
    }
}

/// `-----BEGIN ` and, after it in the same line, `PRIVATE KEY-----`.
fn has_private_key(line: &str) -> bool {
    line.match_indices("-----BEGIN ")
        .any(|(at, marker)| line[at + marker.len()..].contains("PRIVATE KEY-----"))
}

/// A word of the line, cut at spaces and at the characters of `WORD_ENDS`, is an access token.
fn has_access_token(line: &str) -> bool {
    line.split(|c: char| c.is_whitespace() || WORD_ENDS.contains(&c))
        .any(is_access_token)
}

fn is_access_token(word: &str) -> bool {
    has_token_prefix(word) || is_aws_key(word) || is_google_key(word) || is_web_token(word)
}

/// One of the token prefixes, then at least `TOKEN_BODY_MIN` letters, digits, `_` or `-`.
fn has_token_prefix(word: &str) -> bool {
    TOKEN_PREFIXES.iter().any(|prefix| {
        word.strip_prefix(prefix).is_some_and(|rest| {
            rest.chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .count()
                >= TOKEN_BODY_MIN
        })
    })
}

/// An AWS access key ID: `AKIA` or `ASIA`, then exactly 20 capital letters and digits in all.
fn is_aws_key(word: &str) -> bool {
    (word.starts_with("AKIA") || word.starts_with("ASIA"))
        && word.len() == 20
        && word
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

/// A Google API key: `AIza` and 39 characters in all.
fn is_google_key(word: &str) -> bool {
    word.starts_with("AIza") && word.chars().count() == 39
}

/// A JSON Web Token: `eyJ`, then three parts of at least 10 characters separated by two dots.
fn is_web_token(word: &str) -> bool {
    if !word.starts_with("eyJ") {
        return false;
    }
    let parts: Vec<&str> = word.split('.').collect();
    parts.len() == 3 && parts.iter().all(|part| part.chars().count() >= 10)
}

/// A password name (not directly after a letter or digit), then the value: see
/// `is_password_value`.
fn has_password(line: &str) -> bool {
    // ASCII lower case keeps every byte position of the line, so positions found in `lower`
    // are positions in `line`.
    let lower = line.to_ascii_lowercase();
    PASSWORD_NAMES.iter().any(|name| {
        lower.match_indices(name).any(|(at, _)| {
            let after_letter_or_digit = lower[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
            !after_letter_or_digit && is_password_value(&line[at + name.len()..])
        })
    })
}

/// What follows a password name: optional spaces, one optional quote, `=` or `:`, then a value.
fn is_password_value(after_name: &str) -> bool {
    let rest = after_name.trim_start_matches(' ');
    let rest = rest.strip_prefix(['"', '\'']).unwrap_or(rest);
    rest.strip_prefix(['=', ':']).is_some_and(is_secret_value)
}

/// A value is a secret when, trimmed and without surrounding quotes, it is one word of at least
/// 4 characters, and it is not a placeholder: it starts with `<`, `{`, `[`, `$` or `%`, or it
/// consists only of `*`, `x`, `X` and `.`.
fn is_secret_value(value: &str) -> bool {
    let value = value.trim().trim_matches(['"', '\'']);
    value.chars().count() >= 4
        && !value.chars().any(char::is_whitespace)
        && !value.starts_with(['<', '{', '[', '$', '%'])
        && !value.chars().all(|c| matches!(c, '*' | 'x' | 'X' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kinds of the hits of one line.
    fn kinds(line: &str) -> Vec<&'static str> {
        find(line).into_iter().map(|hit| hit.kind).collect()
    }

    /// Twenty characters of the token alphabet.
    const BODY: &str = "abcdefghij0123456789";

    #[test]
    fn a_private_key_header_is_a_hit() {
        assert_eq!(kinds("-----BEGIN RSA PRIVATE KEY-----"), ["private key"]);
        assert_eq!(
            kinds("-----BEGIN OPENSSH PRIVATE KEY-----"),
            ["private key"]
        );
    }

    #[test]
    fn other_headers_and_a_bare_key_name_are_no_hit() {
        for line in [
            "-----BEGIN CERTIFICATE-----",
            "-----BEGIN PUBLIC KEY-----",
            "PRIVATE KEY-----",
            "BEGIN PRIVATE KEY",
        ] {
            assert!(find(line).is_empty(), "{line}");
        }
    }

    #[test]
    fn each_token_prefix_with_twenty_characters_is_a_hit() {
        for prefix in TOKEN_PREFIXES {
            let line = format!("key {prefix}{BODY}");
            assert_eq!(kinds(&line), ["access token"], "{line}");
        }
    }

    #[test]
    fn a_token_prefix_with_nineteen_characters_is_no_hit() {
        for prefix in TOKEN_PREFIXES {
            let line = format!("key {prefix}{}", &BODY[..19]);
            assert!(find(&line).is_empty(), "{line}");
        }
    }

    #[test]
    fn aws_key_ids_are_twenty_capitals_and_digits() {
        assert_eq!(kinds("AKIAQ7WXYZ3JKLMN2PQR"), ["access token"]);
        assert_eq!(kinds("ASIAQ7WXYZ3JKLMN2PQR"), ["access token"]);
        assert!(find("AKIAQ7WXYZ3JKLMN2PQRS").is_empty());
        assert!(find("AKIAQ7WXYZ3JKLMN2PQ").is_empty());
        assert!(find("AKIAq7wxyz3jklmn2pqr").is_empty());
    }

    #[test]
    fn google_api_keys_are_39_characters_starting_with_aiza() {
        let key = format!("AIza{}", "0123456789abcdefghijklmnopqrstuvwxy");
        assert_eq!(key.len(), 39);
        assert_eq!(kinds(&key), ["access token"]);
        assert!(find(&key[..38]).is_empty());
        assert!(find(&format!("{key}z")).is_empty());
    }

    #[test]
    fn a_json_web_token_has_three_parts_of_at_least_ten_characters() {
        let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dBjftJeZ4CVP_mB92K27u";
        assert_eq!(kinds(token), ["access token"]);
        assert!(find("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0").is_empty());
        assert!(find("eyJabc.eyJdefghijkl.mnopqrstuvw").is_empty());
        assert!(find(&format!("{token}.extra")).is_empty());
    }

    #[test]
    fn a_token_is_found_after_a_separator_and_not_inside_a_word() {
        assert_eq!(kinds(&format!("(ghp_{BODY})")), ["access token"]);
        assert_eq!(kinds(&format!("see \"sk-{BODY}\"")), ["access token"]);
        assert_eq!(kinds(&format!("token=ghp_{BODY}")), ["access token"]);
        assert!(find(&format!("notghp_{BODY}")).is_empty());
    }

    #[test]
    fn a_line_gives_at_most_one_hit_with_the_first_kind() {
        assert_eq!(
            kinds(&format!("-----BEGIN PRIVATE KEY----- token=ghp_{BODY}")),
            ["private key"]
        );
        assert_eq!(kinds(&format!("token=ghp_{BODY}")), ["access token"]);
    }

    #[test]
    fn password_names_with_a_value_are_hits() {
        for line in [
            "password: hunter22",
            "Passwd=hunter22",
            "pwd = hunter22",
            "secret: hunter22",
            "token: hunter22",
            "api key: hunter22",
            "api-key: hunter22",
            "api_key: hunter22",
            "apikey: hunter22",
            "password: \"hunter22\"",
            "password = 'hunter22'",
            "password \"= hunter22",
            "db_password: hunter22",
        ] {
            assert_eq!(kinds(line), ["password"], "{line}");
        }
    }

    #[test]
    fn a_name_inside_a_word_or_without_a_separator_is_no_hit() {
        assert!(find("mypassword: hunter22").is_empty());
        assert!(find("mytoken: hunter22").is_empty());
        assert!(find("2pwd: hunter22").is_empty());
        assert!(find("étoken: hunter22").is_empty());
        assert!(find("secretary: hunter22").is_empty());
        assert!(find("tokens: hunter22").is_empty());
        assert!(find("password hunter22").is_empty());
    }

    #[test]
    fn a_value_is_one_word_of_at_least_four_characters() {
        assert_eq!(kinds("password: abcd"), ["password"]);
        assert_eq!(kinds("Größe pwd=hunter22ü"), ["password"]);
        assert!(find("password: abc").is_empty());
        assert!(find("password:").is_empty());
        assert!(find("password: \"\"").is_empty());
        assert!(find("password: \"abc\"").is_empty());
    }

    #[test]
    fn placeholders_are_no_hit() {
        for line in [
            "password: <your password>",
            "password: <secret>",
            "password = ****",
            "password: xxxxxx",
            "password: X.X.X.X",
            "token: $TOKEN",
            "token: %TOKEN%",
            "token: {token}",
            "token: [token]",
        ] {
            assert!(find(line).is_empty(), "{line}");
        }
    }

    #[test]
    fn several_words_are_no_hit_and_one_word_is() {
        assert!(find("password: ask the administrator").is_empty());
        assert_eq!(kinds("password: required"), ["password"]);
    }

    #[test]
    fn field_lines_of_the_tool_are_no_hit() {
        assert!(find("- status: approved").is_empty());
        assert!(find("- tags: token-ring").is_empty());
    }

    #[test]
    fn lines_are_numbered_from_one_and_the_text_is_trimmed() {
        let text = "# Title\n\n  password: hunter22  \nfine\n";
        assert_eq!(
            find(text),
            [Hit {
                line: 3,
                kind: "password",
                text: "password: hunter22".into(),
            }]
        );
    }

    #[test]
    fn a_crlf_line_ending_is_not_part_of_the_text() {
        assert_eq!(
            find("a\r\npassword: hunter22\r\n"),
            [Hit {
                line: 2,
                kind: "password",
                text: "password: hunter22".into(),
            }]
        );
    }

    #[test]
    fn a_new_file_gives_every_hit() {
        assert_eq!(
            added(None, "intro\npassword: hunter22\n", &[]),
            [Hit {
                line: 2,
                kind: "password",
                text: "password: hunter22".into(),
            }]
        );
    }

    #[test]
    fn a_line_the_old_text_already_had_is_not_new() {
        let old = "password: hunter22\n";
        let new = "more text\npassword: hunter22\n";
        assert!(added(Some(old), new, &[]).is_empty());
    }

    #[test]
    fn a_changed_line_is_new() {
        let hits = added(Some("password: hunter22\n"), "password: hunter23\n", &[]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].line, 1);
    }

    #[test]
    fn a_line_whose_hash_is_allowed_is_not_new() {
        let allowed = vec![hash::sha256_hex(b"password: hunter22")];
        assert!(added(None, "password: hunter22\n", &allowed).is_empty());
    }

    #[test]
    fn a_missing_allowed_file_is_an_empty_list() {
        let state = tempfile::tempdir().unwrap();
        assert!(allowed(state.path()).unwrap().is_empty());
    }

    #[test]
    fn allowing_nothing_writes_nothing() {
        let state = tempfile::tempdir().unwrap();
        allow(state.path(), &[]).unwrap();
        assert!(!state.path().join(FILE).exists());
    }

    #[test]
    fn allow_then_allowed_gives_the_hashes_back_sorted_and_once() {
        let state = tempfile::tempdir().unwrap();
        let text = "password: hunter22\nsecret: tr0ub4dor\n";
        let hits = find(text);
        allow(state.path(), &hits).unwrap();
        allow(state.path(), &hits[..1]).unwrap();
        let mut expected: Vec<String> = hits
            .iter()
            .map(|hit| hash::sha256_hex(hit.text.as_bytes()))
            .collect();
        expected.sort();
        let given = allowed(state.path()).unwrap();
        assert_eq!(given, expected);
        assert!(added(None, text, &given).is_empty());
    }

    #[test]
    fn an_allowed_file_that_is_not_an_array_of_strings_is_a_parse_error() {
        let state = tempfile::tempdir().unwrap();
        for text in ["{}", "[1, 2]", "not json"] {
            std::fs::write(state.path().join(FILE), text).unwrap();
            let error = allowed(state.path()).unwrap_err();
            assert_eq!(error.code, Code::Parse, "{text}");
            assert!(error.message.contains(FILE), "{}", error.message);
        }
    }

    #[test]
    fn listed_gives_the_lines_and_kinds_and_never_the_text() {
        let hits = [
            Hit {
                line: 12,
                kind: "password",
                text: "password: hunter22".into(),
            },
            Hit {
                line: 14,
                kind: "private key",
                text: "-----BEGIN RSA PRIVATE KEY-----".into(),
            },
        ];
        assert_eq!(
            listed("DEC-001 first.md", &hits),
            "DEC-001 first.md: line 12 (password); line 14 (private key)"
        );
    }
}
