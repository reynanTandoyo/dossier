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
        1
    }

    /// The name of the code in SCREAMING_SNAKE_CASE, for example `NO_DOSSIER`.
    pub fn name(&self) -> &'static str {
        ""
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::new(Code::Io, error.to_string())
    }
}
