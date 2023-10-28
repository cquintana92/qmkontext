#[derive(Clone, Debug)]
pub enum Error {
    CannotGetCurrentProgram,
    UserConfigExecutionError(String),
    SendError(String),
    HidError(String),
}

impl From<hidapi::HidError> for Error {
    fn from(value: hidapi::HidError) -> Self {
        Error::SendError(format!("hidapi error: {}", value))
    }
}
