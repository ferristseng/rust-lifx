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

impl From<u8> for Service {
  #[inline]
  fn from(b: u8) -> Service {
    use Service::*;

    match b {
      1 => Udp,
      _ => Reserved
    }
  }
}


/// Power level for Device::SetPower and Device::GetPower.
///
#[derive(Debug, Eq, PartialEq)]
pub enum Power {
  Standby,
  Max
}

impl Into<u16> for Power {
  #[inline]
  fn into(self) -> u16 {
    use Power::*;

    match self {
      Standby => 0,
      Max => 65535
    }
  }
}

impl From<u16> for Power {
  #[inline]
  fn from(v: u16) -> Power {
    use Power::*;

    match v {
      0 => Standby,
      _ => Max
    }
  }
}


/// Payload enumeration.
///
#[derive(Debug)]
pub enum Payload {
  Device(Device),
  Light(Light)
}

impl Payload {
  #[inline]
  pub fn typ(&self) -> u16 {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.typ(), 
      Light(ref lightm) => lightm.typ()
    }
  }

  #[inline]
  pub fn tagged(&self) -> bool {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.tagged(),
      Light(ref lightm) => lightm.tagged()
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.size(),
      Light(ref lightm) => lightm.size()
    }
  }

  #[inline]
  pub fn requires_response(&self) -> bool {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.requires_response(),
      Light(ref lightm) => lightm.requires_response()
    }
  }

  pub fn decode<D : Decoder>(d: &mut D, tag: u16) -> Result<Payload, D::Error> {
    match tag {
      2 => 
        Ok(Payload::Device(Device::GetService)),
      3 => 
        {
          let service = From::from(try!(d.read_u8()));
          let port = try!(d.read_u32());

          Ok(Payload::Device(Device::StateService(service, port)))
        }
      12 => 
        Ok(Payload::Device(Device::GetHostInfo)),
      13 =>
        {
          let signal = try!(d.read_f32());
          let tx = try!(d.read_u32());
          let rx = try!(d.read_u32());
          let reserved = try!(d.read_i16());

          Ok(Payload::Device(Device::StateHostInfo(signal, tx, rx, reserved)))
        }
      14 =>
        Ok(Payload::Device(Device::GetHostFirmware)),
      15 =>
        {
          let build = try!(d.read_u64());
          let reserved = try!(d.read_u64());
          let version = try!(d.read_u32());

          Ok(Payload::Device(Device::StateHostFirmware(build, reserved, version)))
        }
      16 =>
        Ok(Payload::Device(Device::GetWifiInfo)),
      17 => 
        {
          let signal = try!(d.read_f32());
          let tx = try!(d.read_u32());
          let rx = try!(d.read_u32());
          let reserved = try!(d.read_i16());

          Ok(Payload::Device(Device::StateWifiInfo(signal, tx, rx, reserved)))
        }
      18 =>
        Ok(Payload::Device(Device::GetWifiFirmware)),
      19 => 
        {
          let build = try!(d.read_u64());
          let reserved = try!(d.read_u64());
          let version = try!(d.read_u32());

          Ok(Payload::Device(Device::StateWifiFirmware(build, reserved, version)))
        }
      20 =>
        Ok(Payload::Device(Device::GetPower)),
      21 => 
        Ok(Payload::Device(Device::SetPower(From::from(try!(d.read_u16()))))),
      22 =>
        Ok(Payload::Device(Device::StatePower(From::from(try!(d.read_u16()))))),
      23 =>
        Ok(Payload::Device(Device::GetLabel)),
      25 => 
        {
          let mut s = Vec::with_capacity(32);

          for _ in 0..32 { 
            let b = try!(d.read_u8());
            if b == 0 { break; }
            s.push(b) 
          }

          unsafe {
            Ok(Payload::Device(Device::StateLabel(String::from_utf8_unchecked(s))))
          }
        },
      32 =>
        Ok(Payload::Device(Device::GetVersion)),
      33 =>
        {
          let vendor = try!(d.read_u32());
          let product = try!(d.read_u32());
          let version = try!(d.read_u32());

          Ok(Payload::Device(Device::StateVersion(vendor, product, version)))
        }
      34 =>
        Ok(Payload::Device(Device::GetInfo)),
      35 =>
        {
          let time = try!(d.read_u64());
          let uptime = try!(d.read_u64());
          let downtime = try!(d.read_u64());

          Ok(Payload::Device(Device::StateInfo(time, uptime, downtime)))
        }
      45 =>
        Ok(Payload::Device(Device::Acknowledgement)),
      116 =>
        Ok(Payload::Light(Light::GetPower)),
      117 =>
        {
          let level = From::from(try!(d.read_u16()));
          let duration = try!(d.read_u32());

          Ok(Payload::Light(Light::SetPower(level, duration)))
        }
      118 => 
        Ok(Payload::Light(Light::StatePower(From::from(try!(d.read_u16()))))),
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
  GetWifiInfo,
  StateWifiInfo(f32, u32, u32, i16),
  GetWifiFirmware,
  StateWifiFirmware(u64, u64, u32),
  GetPower,
  SetPower(Power),
  StatePower(Power),
  GetLabel,
  StateLabel(String),
  GetVersion,
  StateVersion(u32, u32, u32),
  GetInfo,
  StateInfo(u64, u64, u64),
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
      GetWifiInfo => 16,
      StateWifiInfo(_, _, _, _) => 17,
      GetWifiFirmware => 18,
      StateWifiFirmware(_, _, _) => 19,
      GetPower => 20,
      SetPower(_) => 21,
      StatePower(_) => 22,
      GetLabel => 23,
      StateLabel(_) => 25,
      GetVersion => 32,
      StateVersion(_, _, _) => 33,
      GetInfo => 34,
      StateInfo(_, _, _) => 35,
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
      GetService | GetHostInfo | GetHostFirmware | GetWifiInfo | 
      GetWifiFirmware | GetPower | SetPower(_) | GetLabel | GetVersion | 
      GetInfo => true,
      _ => false
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Device::*;

    match *self {
      GetService | GetHostInfo | GetHostFirmware | GetWifiInfo | 
      GetWifiFirmware | GetPower | GetLabel| GetVersion | GetInfo | 
      Acknowledgement => 0,
      SetPower(_) | StatePower(_) => 2,
      StateService(_, _) => 5,
      StateVersion(_, _, _) => 12,
      StateHostInfo(_, _, _, _) | StateWifiInfo(_, _, _, _) => 14,
      StateHostFirmware(_, _, _) | StateWifiFirmware(_, _, _) => 20,
      StateInfo(_, _, _) => 24,
      StateLabel(_) => 32
    }
  }
}


/// Light messages.
///
#[derive(Debug)]
pub enum Light {
  GetPower,
  SetPower(Power, u32),
  StatePower(Power)
}

impl Light {
  #[inline]
  pub fn typ(&self) -> u16 {
    use Light::*;

    match *self {
      GetPower => 116,
      SetPower(_, _) => 117,
      StatePower(_) => 118
    }
  }

  #[inline]
  pub fn tagged(&self) -> bool { false }

  #[inline]
  pub fn requires_response(&self) -> bool {
    use Light::*;

    match *self {
      GetPower | SetPower(_, _) => true,
      _ => false
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Light::*;

    match *self {
      GetPower => 0,
      StatePower(_) => 2,
      SetPower(_, _) => 6
    }
  }
}