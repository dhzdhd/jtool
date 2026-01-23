use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error while parsing JSON {0}")]
    JSONParsing(serde_json::Error),
    #[error("Error while stringifying JSON")]
    JSONStringify,
    #[error("Not implemented yet")]
    NotImplemented,
}
