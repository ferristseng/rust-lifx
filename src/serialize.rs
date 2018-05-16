use std::io::Cursor;
use std::mem;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

/// encodes a series of bytes
///
pub fn encode<T: Encodable>(data: &T) -> Result<Vec<u8>, String> {
  let mut encoder = ByteEncoder::new();
  try!(data.encode(&mut encoder));
  Ok(encoder.bytes)
}

/// decodes a series of bytes
///
pub fn decode<T: Decodable>(data: &[u8]) -> Result<T, String> {
  let mut decoder = ByteDecoder::new(data);
  T::decode(&mut decoder)
}

struct ByteEncoder {
  bytes: Vec<u8>,
}

impl ByteEncoder {
  #[inline(always)]
  fn new() -> ByteEncoder {
    ByteEncoder {
      bytes: Vec::new(),
    }
  }
}

impl Encoder for ByteEncoder {
  type Error = String;

  #[inline(always)]
  fn emit_nil(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn emit_usize(&mut self, v: usize) -> Result<(), Self::Error> {
    if mem::size_of::<usize>() == 4 {
      self.emit_u32(v as u32)
    } else {
      self.emit_u64(v as u64)
    }
  }

  #[inline]
  fn emit_u64(&mut self, v: u64) -> Result<(), Self::Error> {
    self
      .bytes
      .write_u64::<LittleEndian>(v)
      .or(err!("failed to write u64"))
  }

  #[inline]
  fn emit_u32(&mut self, v: u32) -> Result<(), Self::Error> {
    self
      .bytes
      .write_u32::<LittleEndian>(v)
      .or(err!("failed to write u32"))
  }

  #[inline]
  fn emit_u16(&mut self, v: u16) -> Result<(), Self::Error> {
    self
      .bytes
      .write_u16::<LittleEndian>(v)
      .or(err!("failed to write u16"))
  }

  #[inline]
  fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error> {
    self.bytes.write_u8(v).or(err!("failed to write u8"))
  }

  #[inline]
  fn emit_isize(&mut self, v: isize) -> Result<(), Self::Error> {
    if mem::size_of::<isize>() == 4 {
      self.emit_i32(v as i32)
    } else {
      self.emit_i64(v as i64)
    }
  }

  #[inline]
  fn emit_i64(&mut self, v: i64) -> Result<(), Self::Error> {
    self
      .bytes
      .write_i64::<LittleEndian>(v)
      .or(err!("failed to write i64"))
  }

  #[inline]
  fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
    self
      .bytes
      .write_i32::<LittleEndian>(v)
      .or(err!("failed to write i32"))
  }

  #[inline]
  fn emit_i16(&mut self, v: i16) -> Result<(), Self::Error> {
    self
      .bytes
      .write_i16::<LittleEndian>(v)
      .or(err!("failed to write i16"))
  }

  #[inline]
  fn emit_i8(&mut self, v: i8) -> Result<(), Self::Error> {
    self.bytes.write_i8(v).or(err!("failed to write i8"))
  }

  #[inline]
  fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
    self.emit_u8(v as u8).or(err!("failed to write bool"))
  }

  #[inline]
  fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
    self
      .bytes
      .write_f64::<LittleEndian>(v)
      .or(err!("failed to write f64"))
  }

  #[inline]
  fn emit_f32(&mut self, v: f32) -> Result<(), Self::Error> {
    self
      .bytes
      .write_f32::<LittleEndian>(v)
      .or(err!("failed to write f32"))
  }

  #[inline]
  fn emit_char(&mut self, v: char) -> Result<(), Self::Error> {
    let mut buf = [0; 4];
    let encoded = v.encode_utf8(&mut buf[..]);

    for b in encoded.bytes() {
      try!(self.emit_u8(b))
    }

    Ok(())
  }

  #[inline]
  fn emit_str(&mut self, v: &str) -> Result<(), Self::Error> {
    for c in v.chars() {
      try!(self.emit_char(c))
    }

    try!(self.emit_u8(0));

    Ok(())
  }

  #[inline]
  fn emit_enum<F>(&mut self, _: &str, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_enum_variant<F>(
    &mut self,
    _: &str,
    _: usize,
    _: usize,
    f: F,
  ) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_enum_variant_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_enum_struct_variant<F>(
    &mut self,
    _: &str,
    _: usize,
    _: usize,
    f: F,
  ) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_enum_struct_variant_field<F>(
    &mut self,
    _: &str,
    _: usize,
    f: F,
  ) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_struct_field<F>(
    &mut self,
    _: &str,
    _: usize,
    f: F,
  ) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_tuple<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_tuple_struct<F>(
    &mut self,
    _: &str,
    _: usize,
    f: F,
  ) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_tuple_struct_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn emit_option<F>(&mut self, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  fn emit_option_none(&mut self) -> Result<(), Self::Error> {
    self.emit_u8(0)
  }

  fn emit_option_some<F>(&mut self, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    try!(self.emit_u8(1));
    f(self)
  }

  fn emit_seq<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  fn emit_map<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  fn emit_map_elt_key<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }

  fn emit_map_elt_val<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<(), Self::Error>,
  {
    f(self)
  }
}

struct ByteDecoder<'a> {
  cursor: Cursor<&'a [u8]>,
}

impl<'a> ByteDecoder<'a> {
  #[inline(always)]
  fn new(bytes: &[u8]) -> ByteDecoder {
    ByteDecoder {
      cursor: Cursor::new(bytes),
    }
  }

  fn read_char_with_first_byte(&mut self, first: u8) -> Result<char, String> {
    let mut c = 0;
    let len = match first & 0b1111_1100 {
      0b0000_0000 => {
        c |= first as u32;
        1
      }
      0b1100_0000 => {
        c |= (first & 0b0001_1111) as u32;
        2
      }
      0b1110_0000 => {
        c |= (first & 0b0000_1111) as u32;
        3
      }
      0b1111_0000 => {
        c |= (first & 0b0000_0111) as u32;
        4
      }
      0b1111_1000 => {
        c |= (first & 0b0000_0011) as u32;
        5
      }
      0b1111_1100 => {
        c |= (first & 0b0000_0001) as u32;
        6
      }
      _ => panic!("unexpected first byte"),
    };

    for _ in 1..len {
      c <<= 6;
      let byte = try!(self.read_u8()) & 0b0011_1111;
      c |= byte as u32;
    }

    ::std::char::from_u32(c).ok_or("failed to read char".to_string())
  }
}

impl<'a> Decoder for ByteDecoder<'a> {
  type Error = String;

  #[inline(always)]
  fn read_nil(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }

  #[inline]
  fn read_usize(&mut self) -> Result<usize, Self::Error> {
    self
      .cursor
      .read_uint::<LittleEndian>(mem::size_of::<usize>())
      .and_then(|v| Ok(v as usize))
      .or(err!("read usize failed"))
  }

  #[inline]
  fn read_u64(&mut self) -> Result<u64, Self::Error> {
    self
      .cursor
      .read_u64::<LittleEndian>()
      .or(err!("read u64 failed"))
  }

  #[inline]
  fn read_u32(&mut self) -> Result<u32, Self::Error> {
    self
      .cursor
      .read_u32::<LittleEndian>()
      .or(err!("read u32 failed"))
  }

  #[inline]
  fn read_u16(&mut self) -> Result<u16, Self::Error> {
    self
      .cursor
      .read_u16::<LittleEndian>()
      .or(err!("read u16 failed"))
  }

  #[inline]
  fn read_u8(&mut self) -> Result<u8, Self::Error> {
    self.cursor.read_u8().or(err!("read u8 failed"))
  }

  #[inline]
  fn read_isize(&mut self) -> Result<isize, Self::Error> {
    self
      .cursor
      .read_int::<LittleEndian>(mem::size_of::<isize>())
      .and_then(|v| Ok(v as isize))
      .or(err!("read isize failed"))
  }

  #[inline]
  fn read_i64(&mut self) -> Result<i64, Self::Error> {
    self
      .cursor
      .read_i64::<LittleEndian>()
      .or(err!("read i64 failed"))
  }

  #[inline]
  fn read_i32(&mut self) -> Result<i32, Self::Error> {
    self
      .cursor
      .read_i32::<LittleEndian>()
      .or(err!("read i32 failed"))
  }

  #[inline]
  fn read_i16(&mut self) -> Result<i16, Self::Error> {
    self
      .cursor
      .read_i16::<LittleEndian>()
      .or(err!("read i16 failed"))
  }

  #[inline]
  fn read_i8(&mut self) -> Result<i8, Self::Error> {
    self.cursor.read_i8().or(err!("read i8 failed"))
  }

  #[inline]
  fn read_bool(&mut self) -> Result<bool, Self::Error> {
    self
      .read_u8()
      .and_then(|b| Ok(b > 0))
      .or(err!("read bool failed"))
  }

  #[inline]
  fn read_f64(&mut self) -> Result<f64, Self::Error> {
    self
      .cursor
      .read_f64::<LittleEndian>()
      .or(err!("read f64 failed"))
  }

  #[inline]
  fn read_f32(&mut self) -> Result<f32, Self::Error> {
    self
      .cursor
      .read_f32::<LittleEndian>()
      .or(err!("read f32 failed"))
  }

  #[inline]
  fn read_char(&mut self) -> Result<char, Self::Error> {
    let first = try!(self.read_u8());
    self.read_char_with_first_byte(first)
  }

  #[inline]
  fn read_str(&mut self) -> Result<String, Self::Error> {
    let mut s = String::new();

    loop {
      let b = try!(self.read_u8());

      if b == 0b0000_0000 {
        break;
      }

      s.push(try!(self.read_char_with_first_byte(b)));
    }

    Ok(s)
  }

  #[inline]
  fn read_enum<T, F>(&mut self, _: &str, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_enum_variant<T, F>(
    &mut self,
    _: &[&str],
    mut f: F,
  ) -> Result<T, Self::Error>
  where
    F: FnMut(&mut Self, usize) -> Result<T, Self::Error>,
  {
    f(self, 0)
  }

  #[inline]
  fn read_enum_variant_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_enum_struct_variant<T, F>(
    &mut self,
    _: &[&str],
    mut f: F,
  ) -> Result<T, Self::Error>
  where
    F: FnMut(&mut Self, usize) -> Result<T, Self::Error>,
  {
    f(self, 0)
  }

  #[inline]
  fn read_enum_struct_variant_field<T, F>(
    &mut self,
    _: &str,
    _: usize,
    f: F,
  ) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_struct<T, F>(&mut self, _: &str, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_struct_field<T, F>(
    &mut self,
    _: &str,
    _: usize,
    f: F,
  ) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_tuple<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_tuple_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_tuple_struct<T, F>(
    &mut self,
    _: &str,
    _: usize,
    f: F,
  ) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_tuple_struct_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_option<T, F>(&mut self, mut f: F) -> Result<T, Self::Error>
  where
    F: FnMut(&mut Self, bool) -> Result<T, Self::Error>,
  {
    f(self, false)
  }

  #[inline]
  fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>,
  {
    f(self, 0)
  }

  #[inline]
  fn read_seq_elt<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_map<T, F>(&mut self, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>,
  {
    f(self, 0)
  }

  #[inline]
  fn read_map_elt_key<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline]
  fn read_map_elt_val<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
  where
    F: FnOnce(&mut Self) -> Result<T, Self::Error>,
  {
    f(self)
  }

  #[inline(always)]
  fn error(&mut self, s: &str) -> Self::Error {
    s.to_string()
  }
}
