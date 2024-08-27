use crate::capture::Capture;
use std::io;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tracing::{debug, error, warn};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse packet at layer {layer} with protocol {protocol}")]
    PacketParseError { layer: String, protocol: String },
    #[error("Protocol {protocol} on layer {layer} is not implemented yet")]
    UnimplementedError { layer: String, protocol: String },
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    #[error("Pcap error: {0}")]
    Pcap(#[from] pcap::Error),
    #[error("Channel error: {0}")]
    Channel(#[from] Box<SendError<Capture>>),
    #[error("Packet doesn't match with the application")]
    ApplicationParsing,
}

pub fn handle_error(err: Error) {
    match err {
        Error::PacketParseError { .. } => warn!("{}", err),
        Error::UnimplementedError { .. } => debug!("{}", err),
        Error::IO(_) => {
            error!("{}", err);
            panic!("{}", err)
        }
        Error::Channel(_) => {
            error!("{}", err);
            panic!("{}", err)
        }
        Error::Pcap(err) => match err {
            pcap::Error::TimeoutExpired => {
                debug!("{}", err);
            }
            _ => {
                error!("{}", err);
                panic!("{}", err)
            }
        },
        Error::ApplicationParsing => {}
    }
}
