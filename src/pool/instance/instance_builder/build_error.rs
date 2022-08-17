use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("missing parameter {0}")]
    MissingParameter(String)
}