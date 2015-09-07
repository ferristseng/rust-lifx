use header::Header;
use payload::Payload;

use rustc_serialize::{Encoder, Encodable, Decoder, Decodable};


const CLIENT_ID: u32 = 1014;


#[derive(Debug)]
pub struct Message {
  header: Header,
  payload: Payload
}

impl Message {
  pub fn new(msg: Payload, ack_required: bool, target: u64) -> Message {
    Message {
      header: Header::new(
        msg.size() + Header::mem_size(), 
        msg.tagged(),
        CLIENT_ID,
        target,
        ack_required,
        msg.requires_response(),
        0,
        msg.typ()),
      payload: msg
    }
  }
}

impl Encodable for Message {
  fn encode<S : Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    s.emit_struct(
      "Message",
      self.header.size() as usize,
      |mut s| {
        try!(s.emit_struct_field("header", 0, |mut s| self.header.encode(s)));
        Ok(())
      })
  }
}

impl Decodable for Message {
  fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
    d.read_struct(
      "Header",
      0,
      |mut d| {
        let header = try!(d.read_struct_field(
          "header", 
          0, 
          |mut d| Header::decode(d)));
        let message = try!(d.read_struct_field(
          "payload",
          0,
          |mut d| Payload::decode(d, header.typ())));

        Ok(Message { header: header, payload: message })
      })
  }
}