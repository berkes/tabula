use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct AccountError(String);
impl std::error::Error for AccountError {}
impl Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<&str> for AccountError {
    fn from(message: &str) -> Self {
        AccountError(message.to_string())
    }
}
