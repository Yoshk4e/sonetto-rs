use sonettobuf::{CmdId, prost};
use thiserror::Error;
use tokio::io;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Tokio IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Packet error: {0}")]
    Packet(#[from] PacketError),

    #[error("Command error: {0}")]
    Cmd(#[from] CmdError),

    #[error("Serde JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("User is not logged in")]
    NotLoggedIn,

    #[error("Custom error: {0}")]
    Custom(String),

    #[error("Missing Player id")]
    MissingPlayerId,

    #[error("Invalid request")]
    InvalidRequest,

    #[error("Hero not found")]
    HeroNotFound,

    #[error("Insufficient items")]
    InsufficientItems,

    #[error("Insufficient funds")]
    InsufficientCurrency,

    #[error("Banner not found")]
    BannerNotFound,

    #[error("Banner is not yet active")]
    BannerNotYetActive,

    #[error("Banner has expired")]
    BannerExpired,
}

impl From<std::str::Utf8Error> for AppError {
    fn from(e: std::str::Utf8Error) -> Self {
        AppError::Packet(PacketError::Custom(format!("UTF-8 error: {}", e)))
    }
}

impl From<prost::DecodeError> for AppError {
    fn from(err: prost::DecodeError) -> Self {
        AppError::Custom(format!("decode error: {}", err))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Custom(err.to_string())
    }
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum PacketError {
    #[error("Packet length less than header (expected: {0}, actual: {1})")]
    LengthLessThanHeader(usize, usize),

    #[error("Packet length mismatch (expected: {0}, actual: {1})")]
    LengthMismatch(usize, usize),

    #[error("Client packet data decode failed: {0}")]
    ClientPacketDataDecodeFail(#[from] prost::DecodeError),

    #[error("Server packet data decode failed: {0}")]
    ServerPacketDataDecodeFail(prost::DecodeError),

    #[error("Packet error: {0}")]
    Custom(String),
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum CmdError {
    #[error("Unregistered Cmd: {0}")]
    UnregisteredCmd(i16),

    #[error("Unhandled Cmd: {0:?}")]
    UnhandledCmd(CmdId),

    #[error("Received server packet as client request")]
    ServerPacketReceivedAsClient,
}
