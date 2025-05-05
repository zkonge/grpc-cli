use prost::Message;
use prost_reflect::{DynamicMessage, MessageDescriptor};
use tonic::{
    Status,
    codec::{BufferSettings, Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
};

#[derive(Debug, Clone)]
pub struct DynamicProstCodec {
    req: MessageDescriptor,
    resp: MessageDescriptor,
}

impl DynamicProstCodec {
    pub fn new(req: MessageDescriptor, resp: MessageDescriptor) -> Self {
        Self { req, resp }
    }
}

impl Codec for DynamicProstCodec {
    type Encode = DynamicMessage;
    type Decode = DynamicMessage;

    type Encoder = DynamicProstEncoder;
    type Decoder = DynamicProstDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        DynamicProstEncoder {
            _req: self.req.clone(),
            buffer_settings: BufferSettings::default(),
        }
    }

    fn decoder(&mut self) -> Self::Decoder {
        DynamicProstDecoder {
            resp: self.resp.clone(),
            buffer_settings: BufferSettings::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicProstEncoder {
    _req: MessageDescriptor,
    buffer_settings: BufferSettings,
}

impl Encoder for DynamicProstEncoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        item.encode(buf)
            .expect("Message only errors if not enough space");

        Ok(())
    }

    fn buffer_settings(&self) -> BufferSettings {
        self.buffer_settings
    }
}

#[derive(Debug, Clone)]
pub struct DynamicProstDecoder {
    resp: MessageDescriptor,
    buffer_settings: BufferSettings,
}

impl Decoder for DynamicProstDecoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let item = DynamicMessage::decode(self.resp.clone(), buf)
            .map(Option::Some)
            .map_err(from_decode_error)?;

        Ok(item)
    }

    fn buffer_settings(&self) -> BufferSettings {
        self.buffer_settings
    }
}

fn from_decode_error(error: prost::DecodeError) -> Status {
    // Map Protobuf parse errors to an INTERNAL status code, as per
    // https://github.com/grpc/grpc/blob/master/doc/statuscodes.md
    Status::internal(error.to_string())
}
