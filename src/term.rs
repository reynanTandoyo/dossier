//! The terminal: whether a person is at it, and asking one question.
//!
//! `DOSSIER_ANSWERS` (answers separated by `|`) replaces the terminal in tests.

use crate::error::{Code, Error, Result};
use std::io::{BufRead, IsTerminal, Write};
use std::sync::Mutex;

/// The answers of `DOSSIER_ANSWERS`: split on the first use, then used up one by one by `ask`.
static ANSWERS: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// The value of `DOSSIER_ANSWERS`, when it is set.
fn answers_variable() -> Option<String> {
    std::env::var_os("DOSSIER_ANSWERS").map(|value| value.to_string_lossy().into_owned())
}

/// Whether stdin and stdout are terminals. `DOSSIER_ANSWERS` being set counts as yes.
pub fn is_terminal() -> bool {
    answers_variable().is_some()
        || (std::io::stdin().is_terminal() && std::io::stdout().is_terminal())
}

/// Asks for one line. Not a terminal: Code::NeedsTerminal.
pub fn ask(prompt: &str) -> Result<String> {
    if let Some(variable) = answers_variable() {
        return next_answer(&variable);
    }
    if !is_terminal() {
        return Err(Error::new(
            Code::NeedsTerminal,
            "this command must be run by a person at a terminal",
        ));
    }
    let mut stderr = std::io::stderr();
    write!(stderr, "{prompt}")?;
    stderr.flush()?;
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line)? == 0 {
        return Err(Error::new(Code::NeedsTerminal, "no answer given"));
    }
    Ok(line.trim().to_string())
}

/// The first answer not yet used, trimmed. `variable` is split on the first use only.
fn next_answer(variable: &str) -> Result<String> {
    let mut guard = ANSWERS.lock().unwrap_or_else(|e| e.into_inner());
    let answers = guard.get_or_insert_with(|| variable.split('|').map(str::to_string).collect());
    if answers.is_empty() {
        return Err(Error::new(Code::NeedsTerminal, "no more answers"));
    }
    Ok(answers.remove(0).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dossier_answers_stands_in_for_the_terminal() {
        let _guard = crate::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: ENV_LOCK is held, so no other test in this process reads or changes the
        // environment while this one does.
        unsafe { std::env::set_var("DOSSIER_ANSWERS", "a|b") };
        let terminal = is_terminal();
        let first = ask("first? ");
        let second = ask("second? ");
        let third = ask("third? ");
        // SAFETY: ENV_LOCK is still held here.
        unsafe { std::env::remove_var("DOSSIER_ANSWERS") };
        assert!(terminal);
        assert_eq!(first.unwrap(), "a");
        assert_eq!(second.unwrap(), "b");
        let error = third.err().unwrap();
        assert_eq!(error.code, Code::NeedsTerminal);
        assert_eq!(error.message, "no more answers");
    }
}
