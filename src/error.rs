use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0} is required")]
    RequiredParam(String),

    #[error("{0} is not a valid feed order. Must be 'normal' or 'reversed'")]
    InvalidFeedOrder(String),
}

impl Error {
    pub fn required_param(s: &str) -> Error {
        Error::RequiredParam(s.to_string())
    }
}
