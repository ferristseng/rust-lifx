use std::convert::Into;
use std::fmt::{Debug, Formatter, Error};

use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};


/// Max allowable brightness.
///
pub const MAX_BRIGHTNESS: u16 = ::std::u16::MAX;


const MAX_SATURATION: u16 = ::std::u16::MAX;
const DEFAULT_KELVIN: u16 = 3500;


/// Preseeded HSBK values for convenience.
///
#[derive(Copy, Clone, Debug)]
pub enum Color {
  Red,
  Blue,
  Green,
  Violet,
  Yellow,
  White,
}

impl Color {
  /// Converts a Color enum value to a HSBK with varying brightness.
  ///
  #[inline]
  pub fn to_hsbk(self, brightness: u16) -> HSBK {
    use Color::*;

    match self {
      Red => {
        HSBK {
          hue: 0,
          saturation: MAX_SATURATION,
          brightness: brightness,
          kelvin: DEFAULT_KELVIN,
        }
      }
      Blue => {
        HSBK {
          hue: 0,
          saturation: 0,
          brightness: brightness,
          kelvin: 0,
        }
      }
      Green => {
        HSBK {
          hue: 120,
          saturation: MAX_SATURATION,
          brightness: brightness,
          kelvin: DEFAULT_KELVIN,
        }
      }
      Violet => {
        HSBK {
          hue: 0,
          saturation: 0,
          brightness: brightness,
          kelvin: 0,
        }
      }
      Yellow => {
        HSBK {
          hue: 0,
          saturation: 0,
          brightness: brightness,
          kelvin: 0,
        }
      }
      White => {
        HSBK {
          hue: 0,
          saturation: 0,
          brightness: brightness,
          kelvin: 0,
        }
      }
    }
  }
}


/// Labels from a LiFX blub are always 32 byte strings (not null terminated).
/// Decodes a 32 byte string.
///
fn decode_32_byte_str<D: Decoder>(d: &mut D) -> Result<String, D::Error> {
  let mut s = Vec::with_capacity(32);

  for _ in 0..32 {
    let b = try!(d.read_u8());
    if b == 0 {
      break;
    }
    s.push(b)
  }

  unsafe { Ok(String::from_utf8_unchecked(s)) }
}


/// Decodes a 16 byte array.
///
fn decode_16_byte_arr<D: Decoder>(d: &mut D) -> Result<[u8; 16], D::Error> {
  let mut arr = [0; 16];
  for i in 0..16 {
    arr[i] = try!(d.read_u8());
  }
  Ok(arr)
}


/// Decodes a 64 byte array.
///
fn decode_64_byte_arr<D: Decoder>(d: &mut D) -> Result<[u8; 64], D::Error> {
  let mut arr = [0; 64];
  for i in 0..64 {
    arr[i] = try!(d.read_u8());
  }
  Ok(arr)
}


/// Service enumeration.
///
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Service {
  Udp,
  Reserved,
}

impl Into<u8> for Service {
  #[inline]
  fn into(self) -> u8 {
    use Service::*;

    match self {
      Udp => 1,
      Reserved => 5,
    }
  }
}

impl From<u8> for Service {
  #[inline]
  fn from(b: u8) -> Service {
    use Service::*;

    match b {
      1 => Udp,
      _ => Reserved,
    }
  }
}

impl Encodable for Service {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    use Service::*;

    let id = self.clone().into();
    let var = match *self {
      Udp => "Udp",
      Reserved => "Reserved",
    };

    s.emit_enum("Service", |mut s| {
      s.emit_enum_variant(var, id as usize, 0, |mut s| s.emit_u8(id))
    })
  }
}


/// Power level for Device::SetPower and Device::GetPower.
///
#[derive(Debug, Copy, Clone)]
pub enum Power {
  Standby,
  Max,
}

impl Into<u16> for Power {
  #[inline]
  fn into(self) -> u16 {
    use Power::*;

    match self {
      Standby => 0,
      Max => 65535,
    }
  }
}

impl From<u16> for Power {
  #[inline]
  fn from(v: u16) -> Power {
    use Power::*;

    match v {
      0 => Standby,
      _ => Max,
    }
  }
}

impl Encodable for Power {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    use Power::*;

    let id = self.clone().into();
    let var = match *self {
      Standby => "Standby",
      Max => "Max",
    };

    s.emit_enum("Power",
                |s| s.emit_enum_variant(var, id as usize, 0, |s| s.emit_u16(id)))
  }
}


/// HSBK (Hue, Saturation, Brightness, Kelvin)
///
#[derive(RustcEncodable, RustcDecodable, Debug, Copy, Clone)]
pub struct HSBK {
  hue: u16,
  saturation: u16,
  brightness: u16,
  kelvin: u16,
}

impl HSBK {
  pub fn new(h: u16, s: u16, b: u16, k: u16) -> HSBK {
    assert!(k >= 2500 && k <= 9000);

    HSBK {
      hue: h,
      saturation: s,
      brightness: b,
      kelvin: k,
    }
  }
}


/// Payload enumeration.
///
/// # Notes
/// /
///   * This enum is encodable, but not decodable (since it needs the message
///     type which is only present in the header)!
///
#[derive(Debug, RustcEncodable)]
pub enum Payload {
  Device(Device),
  Light(Light),
}

impl Payload {
  #[inline]
  pub fn typ(&self) -> u16 {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.typ(),
      Light(ref lightm) => lightm.typ(),
    }
  }

  #[inline]
  pub fn tagged(&self) -> bool {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.tagged(),
      Light(ref lightm) => lightm.tagged(),
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.size(),
      Light(ref lightm) => lightm.size(),
    }
  }

  #[inline]
  pub fn requires_response(&self) -> bool {
    use Payload::*;

    match *self {
      Device(ref devm) => devm.requires_response(),
      Light(ref lightm) => lightm.requires_response(),
    }
  }

  /// Yes, this is scary huge. All it is doing is decoding the specific payloads
  /// depending on the tag that was in the message header. Some of these messages
  /// technically aren't even sent by a LiFX bulb, but are still implemented
  /// anyways.
  ///
  pub fn decode<D: Decoder>(d: &mut D, tag: u16) -> Result<Payload, D::Error> {
    match tag {
      2 => Ok(Payload::Device(Device::GetService)),
      3 => {
        let service = From::from(try!(d.read_u8()));
        let port = try!(d.read_u32());

        Ok(Payload::Device(Device::StateService(service, port)))
      }
      12 => Ok(Payload::Device(Device::GetHostInfo)),
      13 => {
        let signal = try!(d.read_f32());
        let tx = try!(d.read_u32());
        let rx = try!(d.read_u32());
        let _ = try!(d.read_i16());

        Ok(Payload::Device(Device::StateHostInfo(signal, tx, rx)))
      }
      14 => Ok(Payload::Device(Device::GetHostFirmware)),
      15 => {
        let build = try!(d.read_u64());
        let _ = try!(d.read_u64());
        let version = try!(d.read_u32());

        Ok(Payload::Device(Device::StateHostFirmware(build, version)))
      }
      16 => Ok(Payload::Device(Device::GetWifiInfo)),
      17 => {
        let signal = try!(d.read_f32());
        let tx = try!(d.read_u32());
        let rx = try!(d.read_u32());
        let _ = try!(d.read_i16());

        Ok(Payload::Device(Device::StateWifiInfo(signal, tx, rx)))
      }
      18 => Ok(Payload::Device(Device::GetWifiFirmware)),
      19 => {
        let build = try!(d.read_u64());
        let _ = try!(d.read_u64());
        let version = try!(d.read_u32());

        Ok(Payload::Device(Device::StateWifiFirmware(build, version)))
      }
      20 => Ok(Payload::Device(Device::GetPower)),
      21 => Ok(Payload::Device(Device::SetPower(From::from(try!(d.read_u16()))))),
      22 => Ok(Payload::Device(Device::StatePower(From::from(try!(d.read_u16()))))),
      23 => Ok(Payload::Device(Device::GetLabel)),
      25 => {
        let label = try!(decode_32_byte_str(d));

        Ok(Payload::Device(Device::StateLabel(label)))
      }
      32 => Ok(Payload::Device(Device::GetVersion)),
      33 => {
        let vendor = try!(d.read_u32());
        let product = try!(d.read_u32());
        let version = try!(d.read_u32());

        Ok(Payload::Device(Device::StateVersion(vendor, product, version)))
      }
      34 => Ok(Payload::Device(Device::GetInfo)),
      35 => {
        let time = try!(d.read_u64());
        let uptime = try!(d.read_u64());
        let downtime = try!(d.read_u64());

        Ok(Payload::Device(Device::StateInfo(time, uptime, downtime)))
      }
      45 => Ok(Payload::Device(Device::Acknowledgement)),
      48 => Ok(Payload::Device(Device::GetLocation)),
      50 => {
        let location = try!(decode_16_byte_arr(d));
        let label = try!(decode_32_byte_str(d));
        let updated = try!(d.read_u64());

        Ok(Payload::Device(Device::StateLocation(location, label, updated)))
      }
      51 => Ok(Payload::Device(Device::GetGroup)),
      53 => {
        let group = try!(decode_16_byte_arr(d));
        let label = try!(decode_32_byte_str(d));
        let updated = try!(d.read_u64());

        Ok(Payload::Device(Device::StateGroup(group, label, updated)))
      }
      58 => Ok(Payload::Device(Device::EchoRequest(try!(decode_64_byte_arr(d))))),
      59 => Ok(Payload::Device(Device::EchoResponse(try!(decode_64_byte_arr(d))))),
      101 => Ok(Payload::Light(Light::Get)),
      102 => {
        let _ = try!(d.read_u8());
        let color = try!(HSBK::decode(d));
        let duration = try!(d.read_u32());

        Ok(Payload::Light(Light::SetColor(color, duration)))
      }
      107 => {
        let color = try!(HSBK::decode(d));
        let _ = try!(d.read_i16());
        let power = try!(d.read_u16());
        let label = try!(decode_32_byte_str(d));
        let _ = try!(d.read_u64());

        Ok(Payload::Light(Light::State(color, power, label)))
      }
      116 => Ok(Payload::Light(Light::GetPower)),
      117 => {
        let level = From::from(try!(d.read_u16()));
        let duration = try!(d.read_u32());

        Ok(Payload::Light(Light::SetPower(level, duration)))
      }
      118 => Ok(Payload::Light(Light::StatePower(From::from(try!(d.read_u16()))))),
      _ => Err(d.error("unrecognized message")),
    }
  }
}


/// Device message.
///
#[derive(RustcEncodable)]
pub enum Device {
  GetService,
  StateService(Service, u32),
  GetHostInfo,
  StateHostInfo(f32, u32, u32),
  GetHostFirmware,
  StateHostFirmware(u64, u32),
  GetWifiInfo,
  StateWifiInfo(f32, u32, u32),
  GetWifiFirmware,
  StateWifiFirmware(u64, u32),
  GetPower,
  SetPower(Power),
  StatePower(Power),
  GetLabel,
  StateLabel(String),
  GetVersion,
  StateVersion(u32, u32, u32),
  GetInfo,
  StateInfo(u64, u64, u64),
  Acknowledgement,
  GetLocation,
  StateLocation([u8; 16], String, u64),
  GetGroup,
  StateGroup([u8; 16], String, u64),
  EchoRequest([u8; 64]),
  EchoResponse([u8; 64]),
}

impl Device {
  #[inline]
  pub fn typ(&self) -> u16 {
    use Device::*;

    match *self {
      GetService => 2,
      StateService(_, _) => 3,
      GetHostInfo => 12,
      StateHostInfo(_, _, _) => 13,
      GetHostFirmware => 14,
      StateHostFirmware(_, _) => 15,
      GetWifiInfo => 16,
      StateWifiInfo(_, _, _) => 17,
      GetWifiFirmware => 18,
      StateWifiFirmware(_, _) => 19,
      GetPower => 20,
      SetPower(_) => 21,
      StatePower(_) => 22,
      GetLabel => 23,
      StateLabel(_) => 25,
      GetVersion => 32,
      StateVersion(_, _, _) => 33,
      GetInfo => 34,
      StateInfo(_, _, _) => 35,
      Acknowledgement => 45,
      GetLocation => 48,
      StateLocation(_, _, _) => 50,
      GetGroup => 51,
      StateGroup(_, _, _) => 53,
      EchoRequest(_) => 58,
      EchoResponse(_) => 59,
    }
  }

  #[inline]
  pub fn tagged(&self) -> bool {
    use Device::*;

    match *self {
      GetService => true,
      _ => false,
    }
  }

  #[inline]
  pub fn requires_response(&self) -> bool {
    use Device::*;

    match *self {
      GetService |
      GetHostInfo |
      GetHostFirmware |
      GetWifiInfo |
      GetWifiFirmware |
      GetPower |
      SetPower(_) |
      GetLabel |
      GetVersion |
      GetInfo |
      GetLocation |
      GetGroup |
      EchoRequest(_) => true,
      _ => false,
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Device::*;

    match *self {
      GetService |
      GetHostInfo |
      GetHostFirmware |
      GetWifiInfo |
      GetWifiFirmware |
      GetPower |
      GetLabel |
      GetVersion |
      GetInfo |
      Acknowledgement |
      GetLocation |
      GetGroup => 0,
      SetPower(_) | StatePower(_) => 2,
      StateService(_, _) => 5,
      StateVersion(_, _, _) => 12,
      StateHostInfo(_, _, _) | StateWifiInfo(_, _, _) => 14,
      StateHostFirmware(_, _) | StateWifiFirmware(_, _) => 20,
      StateInfo(_, _, _) => 24,
      StateLabel(_) => 32,
      StateLocation(_, _, _) | StateGroup(_, _, _) => 56,
      EchoRequest(_) | EchoResponse(_) => 64,
    }
  }
}

impl Debug for Device {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    use Device::*;

    match *self {
      GetService => write!(f, "GetService"),
      StateService(serv, port) => write!(f, "StateService({:?}, {})", serv, port),
      GetHostInfo => write!(f, "GetHostInfo"),
      StateHostInfo(signal, tx, rx) => {
        write!(f, "StateHostInfo({}, {}, {})", signal, tx, rx)
      }
      GetHostFirmware => write!(f, "GetHostFirmware"),
      StateHostFirmware(build, version) => {
        write!(f, "StateHostFirmware({}, {})", build, version)
      }
      GetWifiInfo => write!(f, "GetWifiInfo"),
      StateWifiInfo(signal, tx, rx) => {
        write!(f, "StateWifiInfo({}, {}, {})", signal, tx, rx)
      }
      GetWifiFirmware => write!(f, "GetWifiFirmware"),
      StateWifiFirmware(build, version) => {
        write!(f, "StateWifiFirmware({}, {})", build, version)
      }
      GetPower => write!(f, "GetPower"),
      SetPower(pow) => write!(f, "SetPower({:?})", pow),
      StatePower(pow) => write!(f, "StatePower({:?})", pow),
      GetLabel => write!(f, "GetLabel"),
      StateLabel(ref label) => write!(f, "StateLabel({})", label),
      GetVersion => write!(f, "GetVersion"),
      StateVersion(vendor, product, version) => {
        write!(f, "StateVersion({}, {}, {})", vendor, product, version)
      }
      GetInfo => write!(f, "GetInfo"),
      StateInfo(time, uptime, downtime) => {
        write!(f, "StateInfo({}, {}, {})", time, uptime, downtime)
      }
      Acknowledgement => write!(f, "Acknowledgement"),
      GetLocation => write!(f, "GetLocation"),
      StateLocation(_, ref label, updated) => {
        write!(f, "StateLocation([16], {}, {})", label, updated)
      }
      GetGroup => write!(f, "GetGroup"),
      StateGroup(_, ref label, updated) => {
        write!(f, "StateGroup([16], {}, {})", label, updated)
      }
      EchoRequest(_) => write!(f, "EchoRequest([64])"),
      EchoResponse(_) => write!(f, "EchoResponse([64])"),
    }
  }
}


/// Light messages.
///
#[derive(Debug)]
pub enum Light {
  Get,
  SetColor(HSBK, u32),
  State(HSBK, u16, String),
  GetPower,
  SetPower(Power, u32),
  StatePower(Power),
}

impl Light {
  #[inline]
  pub fn typ(&self) -> u16 {
    use Light::*;

    match *self {
      Get => 101,
      SetColor(_, _) => 102,
      State(_, _, _) => 107,
      GetPower => 116,
      SetPower(_, _) => 117,
      StatePower(_) => 118,
    }
  }

  #[inline]
  pub fn tagged(&self) -> bool {
    false
  }

  #[inline]
  pub fn requires_response(&self) -> bool {
    use Light::*;

    match *self {
      Get | GetPower | SetPower(_, _) | SetColor(_, _) => true,
      _ => false,
    }
  }

  #[inline]
  pub fn size(&self) -> u16 {
    use Light::*;

    match *self {
      Get | GetPower => 0,
      StatePower(_) => 2,
      SetPower(_, _) => 6,
      SetColor(_, _) => 13,
      State(_, _, _) => 24,
    }
  }
}

impl Encodable for Light {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    use Light::*;

    s.emit_enum("Light", |s| {
      match *self {
        Get => s.emit_enum_variant("Get", 0, self.size() as usize, |s| s.emit_nil()),
        SetColor(color, power) => {
          s.emit_enum_variant("SetColor", 1, self.size() as usize, |s| {
            try!(s.emit_enum_variant_arg(0, |s| s.emit_u8(0)));
            try!(s.emit_enum_variant_arg(1, |s| color.encode(s)));
            s.emit_enum_variant_arg(2, |s| s.emit_u32(power))
          })
        }
        State(color, power, ref label) => {
          s.emit_enum_variant("State", 2, self.size() as usize, |s| {
            try!(s.emit_enum_variant_arg(0, |s| color.encode(s)));
            try!(s.emit_enum_variant_arg(1, |s| s.emit_u16(0)));
            try!(s.emit_enum_variant_arg(2, |s| s.emit_u16(power)));
            try!(s.emit_enum_variant_arg(3, |s| label.encode(s)));
            s.emit_enum_variant_arg(3, |s| s.emit_u64(0))
          })
        }
        GetPower => {
          s.emit_enum_variant("GetPower", 3, self.size() as usize, |s| s.emit_nil())
        }
        SetPower(level, duration) => {
          s.emit_enum_variant("SetPower", 4, self.size() as usize, |s| {
            try!(s.emit_enum_variant_arg(0, |s| level.encode(s)));
            s.emit_enum_variant_arg(1, |s| s.emit_u32(duration))
          })
        }
        StatePower(level) => {
          s.emit_enum_variant("StatePower",
                              5,
                              self.size() as usize,
                              |s| s.emit_enum_variant_arg(0, |s| level.encode(s)))
        }
      }
    })
  }
}
