use std::convert::Into;

use rustc_serialize::Decoder;


/// Service enumeration.
///
#[derive(Debug, Eq, PartialEq)]
pub enum Service {
  Udp,
  Reserved
}

impl Into<u8> for Service {
  #[inline]
  fn into(self) -> u8 {
    use Service::*;

    match self {
      Udp => 1,
      Reserved => 5
    }
  }
}


/// Payload enumeration.
///
#[derive(Debug)]
pub enum Payload {
  Device(Device)
}

impl Payload {
  #[inline]
  pub fn typ(&self) -> u16 {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.typ(), 
    }
  }

  #[inline]
  pub fn tagged(&self) -> bool {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.tagged()
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.size()
    }
  }

  #[inline]
  pub fn requires_response(&self) -> bool {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.requires_response()
    }
  }

  pub fn decode<D : Decoder>(d: &mut D, tag: u16) -> Result<Payload, D::Error> {
    use Device::*;
    use Service::*;

    match tag {
      2 => 
        Ok(Payload::Device(GetService)),
      3 => 
        {
          let service = match try!(d.read_u8()) {
            1 => Udp,
            _ => Reserved
          };
          let port = try!(d.read_u32());

          Ok(Payload::Device(StateService(service, port)))
        }
      12 => 
        Ok(Payload::Device(GetHostInfo)),
      13 =>
        {
          let signal = try!(d.read_f32());
          let tx = try!(d.read_u32());
          let rx = try!(d.read_u32());
          let reserved = try!(d.read_i16());

          Ok(Payload::Device(StateHostInfo(signal, tx, rx, reserved)))
        }
      14 =>
        Ok(Payload::Device(GetHostFirmware)),
      15 =>
        {
          let build = try!(d.read_u64());
          let reserved = try!(d.read_u64());
          let version = try!(d.read_u32());

          Ok(Payload::Device(StateHostFirmware(build, reserved, version)))
        }
      23 =>
        Ok(Payload::Device(GetLabel)),
      25 => 
        {
          let mut s = Vec::with_capacity(32);

          for _ in 0..32 { 
            let b = try!(d.read_u8());
            if b == 0 { break; }
            s.push(b) 
          }

          unsafe {
            Ok(Payload::Device(StateLabel(String::from_utf8_unchecked(s))))
          }
        }
      45 =>
        Ok(Payload::Device(Acknowledgement)),
      _ => 
        Err(d.error("unrecognized message"))
    }
  } 
}


/// Device message.
///
#[derive(Debug)]
pub enum Device {
  GetService,
  StateService(Service, u32),
  GetHostInfo,
  StateHostInfo(f32, u32, u32, i16),
  GetHostFirmware,
  StateHostFirmware(u64, u64, u32),
  GetLabel,
  StateLabel(String),
  Acknowledgement
}

impl Device {
  #[inline]
  pub fn typ(&self) -> u16 {
    use Device::*;

    match *self {
      GetService => 2,
      StateService(_, _) => 3,
      GetHostInfo => 12,
      StateHostInfo(_, _, _, _) => 13,
      GetHostFirmware => 14,
      StateHostFirmware(_, _, _) => 15,
      GetLabel => 23,
      StateLabel(_) => 25,
      Acknowledgement => 45
    }
  }

  #[inline]
  pub fn tagged(&self) -> bool {
    use Device::*;

    match *self {
      GetService => true,
      _ => false
    }
  }

  #[inline]
  pub fn requires_response(&self) -> bool {
    use Device::*;

    match *self {
      GetService | GetHostInfo | GetLabel => true,
      _ => false
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Device::*;

    match *self {
      GetService | GetHostInfo | GetHostFirmware | GetLabel | 
      Acknowledgement => 0,
      StateService(_, _) => 5,
      StateHostInfo(_, _, _, _) => 14,
      StateHostFirmware(_, _, _) => 20,
      StateLabel(_) => 32
    }
  }
}