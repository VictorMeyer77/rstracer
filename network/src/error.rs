use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse packet at layer {layer} with protocol {protocol} {data:?}")]
    PacketParseError {
        layer: String,
        protocol: String,
        data: Vec<u8>,
    },
    #[error("Protocol {protocol} on layer {layer} is not implemented yet. {data:?}")]
    UnimplementedError {
        layer: String,
        protocol: String,
        data: Vec<u8>,
    },
    #[error("Layer {layer} from protocol {protocol} does not exist.")]
    NoLayerError { layer: String, protocol: String },
}
