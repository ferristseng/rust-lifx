use std::fmt::{Formatter, Debug, Error};

use header::Header;
use payload::Payload;
use rustc_serialize::{Encoder, Encodable, Decoder, Decodable};


const CLIENT_ID: u32 = 1111;


pub struct Message {
  header: Header,
  payload: Payload,
}

impl Message {
  /// creates a new message.
  ///
  pub fn new(msg: Payload, ack_required: bool, target: u64, seq: u8) -> Message {
    Message {
      header: Header::new(msg.size() + Header::mem_size(),
                          msg.tagged(),
                          CLIENT_ID,
                          target,
                          ack_required,
                          msg.requires_response(),
                          seq,
                          msg.typ()),
      payload: msg,
    }
  }

  #[inline(always)]
  pub fn payload(&self) -> &Payload {
    &self.payload
  }

  #[inline(always)]
  pub fn target(&self) -> u64 {
    self.header.target()
  }
}

impl Debug for Message {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    write!(f, "({:#X}) {:?}", self.header.target(), self.payload)
  }
}

impl Encodable for Message {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    s.emit_struct("Message", self.header.size() as usize, |mut s| {
      try!(s.emit_struct_field("header", 0, |mut s| self.header.encode(s)));
      try!(s.emit_struct_field("payload", 1, |mut s| self.payload.encode(s)));
      Ok(())
    })
  }
}

impl Decodable for Message {
  fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
    d.read_struct("Header", 0, |mut d| {
      let header = try!(d.read_struct_field("header", 0, |mut d| Header::decode(d)));
      let message = try!(d.read_struct_field("payload",
                                             0,
                                             |mut d| Payload::decode(d, header.typ())));

      Ok(Message {
        header: header,
        payload: message,
      })
    })
  }
}
