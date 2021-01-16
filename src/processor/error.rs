use thiserror::Error;

#[derive(Error, Debug)]
pub enum XcapeError {
    #[error("Fail to parse arguments: ${0}")]
    InvalidArg(String),
    #[error("Fail to parse expression arguments(-e). arg: ${map:?}, reason: ${reason:?}")]
    InvalidExpressionArg { map: String, reason: String },
    #[error("X connection error. reason: ${0}")]
    XConnectionInitError(String),
}
