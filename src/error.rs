//! Errors: a code that says what went wrong, and a message for the person.

/// Why a command failed. The code gives the exit status and the name used in `--json` output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Usage,
    NotImplemented,
    Refused,
    NotFound,
    Exists,
    NeedsPerson,
    NeedsTerminal,
    UnknownName,
    MissingField,
    BadValue,
    Busy,
    Changed,
    NoDossier,
    NetworkDrive,
    Parse,
    Config,
    Io,
    TooNew,
    TodoMissing,
    Ssh,
}

/// A failure: its code and a message.
#[derive(Debug, Clone)]
pub struct Error {
    pub code: Code,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// A failure with `code` and `message`.
    pub fn new(code: Code, message: impl Into<String>) -> Error {
        Error {
            code,
            message: message.into(),
        }
    }

    /// The exit status of the failure (the table in docs/design.md section 3).
    pub fn exit(&self) -> u8 {
        match self.code {
            Code::Refused
            | Code::NotFound
            | Code::Exists
            | Code::NeedsPerson
            | Code::NeedsTerminal
            | Code::UnknownName
            | Code::MissingField
            | Code::BadValue => 1,
            Code::Usage | Code::NotImplemented => 2,
            Code::Busy | Code::Changed => 3,
            Code::NoDossier
            | Code::NetworkDrive
            | Code::Parse
            | Code::Config
            | Code::Io
            | Code::TooNew => 4,
            Code::TodoMissing | Code::Ssh => 5,
        }
    }

    /// The name of the code in SCREAMING_SNAKE_CASE, for example `NO_DOSSIER`.
    pub fn name(&self) -> &'static str {
        match self.code {
            Code::Usage => "USAGE",
            Code::NotImplemented => "NOT_IMPLEMENTED",
            Code::Refused => "REFUSED",
            Code::NotFound => "NOT_FOUND",
            Code::Exists => "EXISTS",
            Code::NeedsPerson => "NEEDS_PERSON",
            Code::NeedsTerminal => "NEEDS_TERMINAL",
            Code::UnknownName => "UNKNOWN_NAME",
            Code::MissingField => "MISSING_FIELD",
            Code::BadValue => "BAD_VALUE",
            Code::Busy => "BUSY",
            Code::Changed => "CHANGED",
            Code::NoDossier => "NO_DOSSIER",
            Code::NetworkDrive => "NETWORK_DRIVE",
            Code::Parse => "PARSE",
            Code::Config => "CONFIG",
            Code::Io => "IO",
            Code::TooNew => "TOO_NEW",
            Code::TodoMissing => "TODO_MISSING",
            Code::Ssh => "SSH",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::new(Code::Io, error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [(Code, u8, &str); 20] = [
        (Code::Usage, 2, "USAGE"),
        (Code::NotImplemented, 2, "NOT_IMPLEMENTED"),
        (Code::Refused, 1, "REFUSED"),
        (Code::NotFound, 1, "NOT_FOUND"),
        (Code::Exists, 1, "EXISTS"),
        (Code::NeedsPerson, 1, "NEEDS_PERSON"),
        (Code::NeedsTerminal, 1, "NEEDS_TERMINAL"),
        (Code::UnknownName, 1, "UNKNOWN_NAME"),
        (Code::MissingField, 1, "MISSING_FIELD"),
        (Code::BadValue, 1, "BAD_VALUE"),
        (Code::Busy, 3, "BUSY"),
        (Code::Changed, 3, "CHANGED"),
        (Code::NoDossier, 4, "NO_DOSSIER"),
        (Code::NetworkDrive, 4, "NETWORK_DRIVE"),
        (Code::Parse, 4, "PARSE"),
        (Code::Config, 4, "CONFIG"),
        (Code::Io, 4, "IO"),
        (Code::TooNew, 4, "TOO_NEW"),
        (Code::TodoMissing, 5, "TODO_MISSING"),
        (Code::Ssh, 5, "SSH"),
    ];

    #[test]
    fn every_code_has_the_exit_status_of_the_table() {
        for (code, exit, _) in ALL {
            assert_eq!(Error::new(code, "x").exit(), exit, "{code:?}");
        }
    }

    #[test]
    fn names_are_screaming_snake_case() {
        for (code, _, name) in ALL {
            assert_eq!(Error::new(code, "x").name(), name, "{code:?}");
            assert!(
                !name.is_empty()
                    && !name.starts_with('_')
                    && !name.ends_with('_')
                    && !name.contains("__")
                    && name
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
                "{name} is not SCREAMING_SNAKE_CASE"
            );
        }
    }

    #[test]
    fn name_of_no_dossier_is_no_dossier() {
        assert_eq!(Error::new(Code::NoDossier, "x").name(), "NO_DOSSIER");
    }

    #[test]
    fn name_of_todo_missing_is_todo_missing() {
        assert_eq!(Error::new(Code::TodoMissing, "x").name(), "TODO_MISSING");
    }

    #[test]
    fn name_of_bad_value_is_bad_value() {
        assert_eq!(Error::new(Code::BadValue, "x").name(), "BAD_VALUE");
    }

    #[test]
    fn io_error_becomes_io_code_with_its_text() {
        let error = Error::from(std::io::Error::other("disk is full"));
        assert_eq!(error.code, Code::Io);
        assert_eq!(error.message, "disk is full");
        assert_eq!(error.exit(), 4);
        assert_eq!(error.name(), "IO");
    }

    #[test]
    fn display_prints_the_message() {
        let error = Error::new(Code::Refused, "not allowed here");
        assert_eq!(error.to_string(), "not allowed here");
    }
}
