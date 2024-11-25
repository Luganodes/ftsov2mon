use actix_web::{http::StatusCode, ResponseError};

#[derive(Debug, thiserror::Error)]
pub enum MonError {
    #[error("Error while getting response: {0}")]
    ResponseError(#[from] anyhow::Error),

    #[error("Deserialization Error: {0}")]
    DeserializationError(#[source] anyhow::Error),

    #[error("Something went wrong internall")]
    InternalServerError,

    #[error("Register error")]
    RegisterError(#[source] anyhow::Error),

    #[error("Error while encoding metric families")]
    EncodeError(#[source] anyhow::Error),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("RPC Client Error: {0}")]
    RpcClientError(#[source] anyhow::Error),

    #[error("Couldn't unwrap SyncInfo")]
    UnableToUnwrapSyncInfo,

    #[error("Couldn't convert to hex")]
    ConversionError(#[source] anyhow::Error),
}

impl ResponseError for MonError {
    fn status_code(&self) -> StatusCode {
        match self {
            MonError::ResponseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::DeserializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::RegisterError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::EncodeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::RpcClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::UnableToUnwrapSyncInfo => StatusCode::INTERNAL_SERVER_ERROR,
            MonError::ConversionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<web3::Error> for MonError {
    fn from(value: web3::Error) -> Self {
        match value {
            web3::Error::Unreachable => {
                MonError::RpcClientError(anyhow::anyhow!("RPC is unreachable!"))
            }
            web3::Error::Decoder(s) => {
                MonError::RpcClientError(anyhow::anyhow!("Decoding error: {s:?}"))
            }
            web3::Error::InvalidResponse(s) => {
                MonError::RpcClientError(anyhow::anyhow!("Invalid response from RPC: {s:?}"))
            }
            web3::Error::Transport(s) => {
                MonError::RpcClientError(anyhow::anyhow!("RPC Transport Error: {s:?}"))
            }
            web3::Error::Rpc(s) => MonError::RpcClientError(anyhow::anyhow!("RPC Error: {s:?}")),
            web3::Error::Io(s) => MonError::RpcClientError(anyhow::anyhow!("RPC IO Error: {s:?}")),
            web3::Error::Recovery(s) => {
                MonError::RpcClientError(anyhow::anyhow!("RPC Recovery Error: {s:?}"))
            }
            web3::Error::Internal => {
                MonError::RpcClientError(anyhow::anyhow!("Internal RPC Error"))
            }
        }
    }
}
