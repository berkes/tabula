use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Error(String);
impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error(message.to_string())
    }
}
