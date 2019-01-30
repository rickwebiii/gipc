use ::protobuf::{Message, parse_from_bytes};
use super::error::{Error};

use std::fmt::{Debug};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    /// The start of a GPIC message.
    RequestStart = 0x0,
    /// This message contains more data on the stream.
    Continuation = 0x1,
    /// This message contains no data and the stream is complete.
    EndOfStream = 0x2,
    /// The client is requesting to terminate the channel.
    EndOfChannel = 0x3,
}

impl OpCode {
    pub fn from_u8(val: u8) -> Result<OpCode, Error> {
        let request_start = OpCode::RequestStart as u8;
        let continuation = OpCode::RequestStart as u8;
        let end_of_stream = OpCode::RequestStart as u8;
        let end_of_channel = OpCode::RequestStart as u8;

        match val {
            request_start => Ok(OpCode::RequestStart),
            continuation => Ok(OpCode::Continuation),
            end_of_stream => Ok(OpCode::EndOfStream),
            end_of_channel => Ok(OpCode::EndOfChannel),
            _ => Err(Error::InvalidOpCode(val))
        }
    }
}

#[derive(Debug)]
pub struct GipcMessage<T: Message> {
    op_code: OpCode,
    data: Option<T>,
}

impl <T: Message> GipcMessage<T> {
    pub fn from_bytes(data: &[u8]) -> Result<GipcMessage<T>, Error> {
        let (op_code, data) = data.split_at(1);
        let op_code = OpCode::from_u8(op_code[0])?;

        let data = match op_code {
            OpCode::RequestStart => {
                Some(parse_from_bytes(data)?)
            },
            OpCode::Continuation => {
                Some(parse_from_bytes(data)?)           
            },
            OpCode::EndOfStream => {
                if data.len() > 0 {
                    return Err(Error::OpCodeShouldHaveNoData);
                }

                None
            },
            OpCode::EndOfChannel => {
                if data.len() > 0 {
                    return Err(Error::OpCodeShouldHaveNoData);
                }

                None
            }
        };

        Ok(GipcMessage {
            op_code: op_code,
            data: data,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes: Vec<u8> = vec![];

        bytes.push(self.op_code as u8);

        match &self.data {
            Some(x) => {
                let payload_bytes = x.write_to_bytes()?;

                for i in payload_bytes {
                    bytes.push(i);
                }
            },
            None => { }
        }

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::HelloRequest;
    use super::OpCode;
    use super::GipcMessage;

    #[test]
    pub fn can_serialize_and_deserialize_messages() {
        let mut payload = HelloRequest::new();
        payload.set_name("moo horses".to_owned());

        let message = GipcMessage {
            op_code: OpCode::RequestStart,
            data: Some(payload),
        };

        let bytes = message.to_bytes().unwrap();

        println!("{:?}", bytes);

        let message: GipcMessage<HelloRequest> = GipcMessage::from_bytes(bytes.as_slice()).unwrap();

        assert_eq!(message.data.unwrap().get_name(), "moo horses");
    }
}