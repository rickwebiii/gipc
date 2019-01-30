use ::protobuf::error::{ProtobufError};

#[derive(Debug)]
pub enum Error {
    InvalidOpCode(u8),
    OpCodeShouldHaveNoData,
    ProtobufError(ProtobufError),
}

impl From<ProtobufError> for Error {
    fn from(err: ProtobufError) -> Self {
        Error::ProtobufError(err)
    }
}
