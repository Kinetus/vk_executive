use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BuildError {
    #[error("missing parameter {0}")]
    MissingParameter(String)
}