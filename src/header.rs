use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

#[derive(Debug, Eq, PartialEq)]
pub struct Header {
  size: u16,
  origin: u8,
  tagged: bool,
  addressable: bool,
  protocol: u16,
  source: u32,
  target: u64,
  ack_required: bool,
  res_required: bool,
  sequence: u8,
  typ: u16,
}

impl Header {
  #[inline]
  pub fn new(
    size: u16,
    tagged: bool,
    source: u32,
    target: u64,
    ack_required: bool,
    res_required: bool,
    sequence: u8,
    typ: u16,
  ) -> Header {
    Header {
      size: size,
      origin: 0,
      tagged: tagged,
      addressable: true,
      protocol: 1024,
      source: source,
      target: target,
      ack_required: ack_required,
      res_required: res_required,
      sequence: sequence,
      typ: typ,
    }
  }

  #[inline(always)]
  pub fn target(&self) -> u64 {
    self.target
  }

  #[inline(always)]
  pub fn typ(&self) -> u16 {
    self.typ
  }

  #[inline(always)]
  pub fn size(&self) -> u16 {
    self.size
  }

  #[inline(always)]
  pub fn mem_size() -> u16 {
    36
  }
}

impl Default for Header {
  fn default() -> Header {
    Header::new(0, true, 0, 0, true, true, 0, 0)
  }
}

impl Encodable for Header {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    s.emit_struct("Header", 36, |mut s| {
      // FRAME
      try!(s.emit_struct_field("size", 0, |mut s| s.emit_u16(self.size)));
      try!(
        s.emit_struct_field("origin_tagged_addressable_protocol", 1, |mut s| {
          let mut value = self.origin as u16;
          if self.tagged {
            value |= 0b0010_0000_0000_0000;
          }
          if self.addressable {
            value |= 0b0001_0000_0000_0000;
          }
          s.emit_u16(self.protocol | (value as u16))
        })
      );
      try!(s.emit_struct_field("source", 2, |mut s| s.emit_u32(self.source)));

      // FRAME ADDRESS
      try!(s.emit_struct_field("target", 3, |mut s| s.emit_u64(self.target)));
      try!(
        s.emit_struct_field("res0", 4, |mut s| s.emit_seq(6, |mut s| {
          for i in 0..6 {
            try!(s.emit_seq_elt(i, |mut s| s.emit_u8(0)))
          }
          Ok(())
        }))
      );
      try!(s.emit_struct_field("res1_ackreq_resreq", 5, |mut s| {
        let mut value: u8 = 0;
        if self.ack_required {
          value |= 0b0000_0010;
        }
        if self.res_required {
          value |= 0b0000_0001;
        }
        s.emit_u8(value)
      }));
      try!(s.emit_struct_field("sequence", 6, |mut s| s.emit_u8(self.sequence)));

      // PROTOCOL HEADER
      try!(s.emit_struct_field("res2", 7, |mut s| s.emit_u64(0)));
      try!(s.emit_struct_field("type", 8, |mut s| s.emit_u16(self.typ)));
      try!(s.emit_struct_field("res3", 9, |mut s| s.emit_u16(0)));

      Ok(())
    })
  }
}

impl Decodable for Header {
  fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
    let mut header: Header = Default::default();

    try!(d.read_struct("Header", 36, |mut d| {
      // FRAME
      try!(
        d.read_struct_field("size", 0, |mut d| d.read_u16().and_then(|v| {
          header.size = v;
          Ok(())
        }))
      );
      try!(
        d.read_struct_field(
          "origin_tagged_addressable_protocol",
          1,
          |mut d| d.read_u16().and_then(|v| {
            header.origin = ((v & 0b1100_0000_0000_0000) >> 14) as u8;
            header.tagged = (v & 0b0010_0000_0000_0000) > 0;
            header.addressable = (v & 0b001_0000_0000_0000) > 0;
            header.protocol = v & 0b0000_1111_1111_1111;
            Ok(())
          })
        )
      );
      try!(
        d.read_struct_field("source", 2, |mut d| d.read_u32().and_then(|v| {
          header.source = v;
          Ok(())
        }))
      );

      // FRAME ADDRESS
      try!(
        d.read_struct_field("target", 3, |mut d| d.read_u64().and_then(|v| {
          header.target = v;
          Ok(())
        }))
      );
      try!(
        d.read_struct_field("res0", 4, |mut d| d.read_seq(|mut d, _| {
          for i in 0..6 {
            try!(d.read_seq_elt(i, |mut d| d.read_u8()));
          }
          Ok(())
        }))
      );
      try!(d.read_struct_field("res1_ackreq_resreq", 5, |mut d| {
        d.read_u8().and_then(|v| {
          header.ack_required = v & 0b0000_0010 > 0;
          header.res_required = v & 0b0000_0001 > 0;
          Ok(())
        })
      }));
      try!(
        d.read_struct_field("sequence", 6, |mut d| d.read_u8().and_then(|v| {
          header.sequence = v;
          Ok(())
        }))
      );

      // PROTOCOL HEADER
      try!(d.read_struct_field("res2", 7, |mut d| d.read_u64()));
      try!(
        d.read_struct_field("type", 8, |mut d| d.read_u16().and_then(|v| {
          header.typ = v;
          Ok(())
        }))
      );
      try!(d.read_struct_field("res3", 9, |mut d| d.read_u16()));

      Ok(())
    }));

    Ok(header)
  }
}

#[test]
fn test_header_encode_correctness() {
  use serialize;

  let correct = [
    0x24, 0x0, 0x0, 0x34, 0x29, 0xb9, 0x36, 0xa9, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0, 0x2, 0x0, 0x0, 0x0,
  ];

  let header: Header = Header::new(36, true, 2838935849, 0, false, true, 0, 2);

  assert_eq!(&correct[..], &serialize::encode(&header).unwrap()[..])
}

#[test]
fn test_encode_decode_json() {
  use rustc_serialize::json;

  let header: Header = Header::new(128, true, 256, 1000, false, false, 1, 12);
  let encode = json::as_pretty_json(&header).to_string();
  let decode: Header = json::decode(&encode[..]).unwrap();

  assert_eq!(decode, header);
}

#[test]
fn test_encode_decode_serializer() {
  use serialize;

  let header: Header = Header::new(128, true, 256, 1000, false, false, 1, 12);
  let encode = serialize::encode(&header).unwrap();
  let decode: Header = serialize::decode(&encode[..]).unwrap();

  assert_eq!(decode, header);
}
