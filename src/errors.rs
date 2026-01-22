use std::fmt;

#[derive(Debug)]
pub enum AppError {
    MissingConfig(String),
    InvalidInput(String),
    GitState(String),
    Network(String),
    Audit(String),
    Io(std::io::Error),
    Other(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::MissingConfig(_) => 2,
            AppError::InvalidInput(_) => 3,
            AppError::GitState(_) => 4,
            AppError::Network(_) => 5,
            AppError::Audit(_) => 6,
            AppError::Io(_) | AppError::Other(_) => 1,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingConfig(message)
            | AppError::InvalidInput(message)
            | AppError::GitState(message)
            | AppError::Network(message)
            | AppError::Audit(message)
            | AppError::Other(message) => write!(f, "{message}"),
            AppError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}
