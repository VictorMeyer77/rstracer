use std::io;
use thiserror::Error;
use tokio::task::JoinError;
use tracing::error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Protocol {protocol} on layer {layer} is not implemented yet")]
    UnimplementedError { layer: String, protocol: String },
    #[error("Packet can't be read yet")]
    PacketParsing,
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Pcap error: {0}")]
    Pcap(#[from] pcap::Error),
    #[error("Join error: {0}")]
    Join(#[from] JoinError),
}
