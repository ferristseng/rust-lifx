use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{Ordering, AtomicBool, AtomicUsize, ATOMIC_USIZE_INIT};
use std::ops::{Deref, Drop};
use std::fmt::{Display, Debug, Formatter, Error};
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};

use serialize;
use message::Message;
use payload::{Service, Payload, Device, Light};
use net2::{UdpBuilder, UdpSocketExt};


pub const MESSAGE_INTERVAL: u8 = 50;


/// udp broadcast ip address and lifx default port.
///
static BROADCAST_IP: &'static str = "255.255.255.255:56700";


/// sequence number counter used to confirm acks.
///
static SEQUENCE_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;


/// returns the next sequence number (global, shared counter).
///
fn next_sequence() -> u8 {
  SEQUENCE_COUNTER.fetch_add(1, Ordering::SeqCst) as u8
}


/// sends a message to the specified address.
///
fn send_msg<S: Deref<Target = UdpSocket>, A: ToSocketAddrs>
  (socket: &S,
   addr: A,
   payload: Payload,
   ack_required: bool,
   target: u64)
   -> Result<u8, String> {
  let seq = next_sequence();
  let msg = Message::new(payload, ack_required, target, seq);
  let encoded = try!(serialize::encode(&msg).or(err!("failed to encode")));
  let bytes = try!(socket.send_to(&encoded[..], addr)
                         .or(err!("failed to send message")));

  debug!(target: "device.out", "    Sending: {:?}", msg);

  if bytes == encoded.len() {
    Ok(seq)
  } else {
    err!("wrong number of bytes written")
  }
}


bitflags! {
  pub flags DiscoverOptions: u8 {
    const GET_LABEL         = 0b0000_0001,
    const GET_WIFI          = 0b0000_0010,
    const GET_LOCATION      = 0b0000_0100,
    const GET_HOST_FIRMWARE = 0b0000_1000,
    const GET_GROUP         = 0b0001_0000,
    const GET_POWER         = 0b0010_0000,
    const GET_HOST_INFO     = 0b0100_0000,
    const GET_ALL           = GET_LABEL.bits | GET_WIFI.bits |
                              GET_LOCATION.bits | GET_HOST_FIRMWARE.bits |
                              GET_GROUP.bits | GET_POWER.bits |
                              GET_HOST_INFO.bits
  }
}


/// a bulb is a LiFX device where the service is Udp.
///
#[derive(Clone)]
pub struct Bulb<A: ToSocketAddrs> {
  label: Option<String>,
  location: Option<String>,
  ip: A,
  port: u32,
  target: u64,
  socket: Arc<UdpSocket>,
}

impl<A> Bulb<A> where A: ToSocketAddrs
{
  /// returns the label of the bulb, if one was received.
  ///
  pub fn label(&self) -> Option<&str> {
    match self.label {
      Some(ref label) => Some(&label[..]),
      None => None,
    }
  }

  /// sends a message to this bulb.
  ///
  pub fn send_msg(&self,
                  payload: Payload,
                  ack_required: bool)
                  -> Result<u8, String> {
    send_msg(&self.socket, &self.ip, payload, ack_required, self.target)
  }

  /// sends a message to this bulb, and waits the recommended amount of time.
  ///
  pub fn send_msg_and_wait(&self,
                           payload: Payload,
                           ack_required: bool)
                           -> Result<u8, String> {
    let res = self.send_msg(payload, ack_required);
    thread::sleep(Duration::from_millis(MESSAGE_INTERVAL as u64));
    res
  }
}

impl<A> Display for Bulb<A> where A: ToSocketAddrs + Display
{
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    write!(f,
           "'{:?}' ({}:{} {})",
           self.label(),
           self.ip,
           self.port,
           self.target)
  }
}

impl<A> Debug for Bulb<A> where A: ToSocketAddrs + Debug
{
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    write!(f,
           "Bulb ({}):\nLabel: '{:?}'\nAddr.: {:?}:{}",
           self.target,
           self.label(),
           self.ip,
           self.port)
  }
}


/// the client handles device messages from from any lifx bulb.
///
pub struct Client {
  closed: Arc<AtomicBool>,
  socket: Arc<UdpSocket>,
  devices: Arc<RwLock<HashMap<u64, Bulb<SocketAddr>>>>,
}

impl Client {
  /// creates a new client that will read responses from any lifx bulb.
  ///
  pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Client, String> {
    let closed = Arc::new(AtomicBool::new(false));
    let devices = Arc::new(RwLock::new(HashMap::new()));
    let udp_builder = try!(UdpBuilder::new_v4()
                             .or(err!("failed to create builder")));
    let udp_socket = Arc::new(try!(udp_builder.bind(addr)
                                              .or(err!("failed to bind to addr"))));

    try!(udp_socket.set_read_timeout_ms(Some(500))
                   .or(err!("failed to set read timeout")));
    try!(udp_socket.set_write_timeout_ms(Some(500))
                   .or(err!("failed to set write timeout")));

    let client = Client {
      closed: closed,
      socket: udp_socket,
      devices: devices,
    };

    Ok(client)
  }

  /// listens for certain messages, and updates the client object accordingly
  ///
  pub fn listen(&self) -> JoinHandle<()> {
    let socket = self.socket.clone();
    let closed = self.closed.clone();
    let devices = self.devices.clone();

    thread::spawn(move || {
      let mut buf = [0; 256];

      while !closed.load(Ordering::SeqCst) {
        let (amt, src) = match socket.recv_from(&mut buf[..]) {
          Ok(received) => received,
          Err(_) => continue,
        };
        let resp = serialize::decode::<Message>(&buf[..amt]).unwrap();
        let (payload, target) = resp.unpack();

        macro_rules! update_device_property(
          ($prop:ident, $val:expr) => (
            if let Some(bulb) = devices.write().unwrap().get_mut(&target) {
              bulb.$prop = $val;
            }
          )
        );

        match payload {
          Payload::Device(Device::StateService(Service::Udp, port)) => {
            info!(target: "device.in", "Received device with port: {}", port);

            devices.write().unwrap().entry(target).or_insert(Bulb {
              label: None,
              location: None,
              ip: src,
              port: port,
              target: target,
              socket: socket.clone(),
            });

            info!(target: "device.in", "Devices:");

            for d in devices.read().unwrap().values() {
              info!(target: "device.in", "  Devices: {:?}", d);
            }
          }
          Payload::Device(Device::StateLabel(label)) => {
            info!(
                target: "device.in",
                "Received device label: '{:?}' for {:#X}",
                label,
                target);

            update_device_property!(label, Some(label.clone()));
          }
          Payload::Device(Device::StateLocation(_, location, _)) => {
            info!(
                target: "device.in",
                "Received location label: '{:?}' for {:#X}",
                location,
                target);

            update_device_property!(location, Some(location.clone()));
          }
          Payload::Light(Light::State(color, power, label)) => {
            info!(
              target: "device.in",
              "Received light state: '{:?} (Power {})' for '{}'",
              color,
              power,
              label);
          }
          _ => (),
        }
      }
    })
  }

  /// broadcasts messages to the client at a set interval. use `listen` to
  /// have the client process certain messages.
  ///
  pub fn discover(&self, wait: u64, options: DiscoverOptions) -> JoinHandle<()> {
    use Device::*;

    let socket = self.socket.clone();
    let closed = self.closed.clone();
    let devices = self.devices.clone();

    thread::spawn(move || {
      while !closed.load(Ordering::SeqCst) {
        // TODO: Technically, there should be a LOCK on the socket here. Messages
        // should not be able to be sent between the time the socket is set to
        // broadcast.
        let _ = socket.set_broadcast(true);
        let _ = send_msg(&socket,
                         BROADCAST_IP,
                         Payload::Device(GetService),
                         false,
                         0);
        let _ = socket.set_broadcast(false);

        for d in devices.read().unwrap().values() {
          if !(options & GET_LABEL).is_empty() {
            let _ = d.send_msg_and_wait(Payload::Device(Device::GetLabel), false);
          }

          if !(options & GET_POWER).is_empty() {
            let _ = d.send_msg_and_wait(Payload::Device(Device::GetPower), false);
          }

          if !(options & GET_LOCATION).is_empty() {
            let _ = d.send_msg_and_wait(Payload::Device(Device::GetLocation), false);
          }

          if !(options & GET_GROUP).is_empty() {
            let _ = d.send_msg_and_wait(Payload::Device(Device::GetGroup), false);
          }

          if !(options & GET_HOST_INFO).is_empty() {
            let _ = d.send_msg_and_wait(Payload::Device(Device::GetHostInfo), false);
          }

          if !(options & GET_HOST_FIRMWARE).is_empty() {
            let _ = d.send_msg_and_wait(Payload::Device(Device::GetHostFirmware),
                                        false);
          }

          if !(options & GET_WIFI).is_empty() {
            let _ = d.send_msg_and_wait(Payload::Device(Device::GetWifiFirmware),
                                        false);
          }
        }

        thread::sleep(Duration::from_millis(wait));
      }
    })
  }

  /// sends a message to the specified address.
  ///
  pub fn send_msg<A: ToSocketAddrs>(&self,
                                    addr: A,
                                    payload: Payload,
                                    ack_required: bool,
                                    target: u64)
                                    -> Result<u8, String> {
    send_msg(&self.socket, addr, payload, ack_required, target)
  }

  /// returns a snapshot of the devices that the client has found.
  ///
  pub fn devices(&self) -> HashMap<u64, Bulb<SocketAddr>> {
    self.devices.read().unwrap().deref().clone()
  }

  /// returns a snapshot of a particular device, given its target id.
  ///
  pub fn device(&self, target: u64) -> Option<Bulb<SocketAddr>> {
    match self.devices.read() {
      Ok(devices) => devices.get(&target).map(|d| d.clone()),
      Err(_) => None,
    }
  }

  /// closes a client. it will no longer receive responses from the socket.
  ///
  #[inline(always)]
  pub fn close(&self) {
    self.closed.store(true, Ordering::SeqCst)
  }

  /// checks if a client is closed.
  ///
  #[inline(always)]
  pub fn is_closed(&self) -> bool {
    self.closed.load(Ordering::SeqCst)
  }
}

impl Drop for Client {
  fn drop(&mut self) {
    self.close();
  }
}


#[test]
fn test_sequence_counter_overflow() {
  use std::u8;

  assert_eq!(0, next_sequence());
  for _ in 1..(u8::MAX as usize) + 1 {
    next_sequence();
  }
  assert_eq!(0, next_sequence());
  for _ in 1..(u8::MAX as usize) + 1 {
    next_sequence();
  }
  assert_eq!(0, next_sequence());
}
