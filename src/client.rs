use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{Ordering, AtomicBool, AtomicUsize, 
  ATOMIC_USIZE_INIT};
use std::ops::{Deref, Drop};
use std::fmt::{Display, Formatter, Error};
use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};

use serialize;
use message::Message;
use payload::{Service, Payload, Device};
use net2::{UdpBuilder, UdpSocketExt};


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
fn send_msg<S : Deref<Target = UdpSocket>, A : ToSocketAddrs>(
  socket: &S, 
  addr: A,
  payload: Payload, 
  ack_required: bool, 
  target: u64
) -> Result<u8, String> 
{
  let seq = next_sequence();
  let msg = Message::new(payload, ack_required, target, seq);
  let encoded = try!(serialize::encode(&msg).or(err!("failed to encode")));
  let bytes = try!(
    socket.send_to(&encoded[..], addr).or(err!("failed to send message")));

  println!("    Sending: {:?}", msg);

  if bytes == encoded.len() {
    Ok(seq)
  } else {
    err!("wrong number of bytes written")
  }
}


/// a bulb is a LiFX device where the service is Udp. 
///
#[derive(Clone)]
pub struct Bulb<A : ToSocketAddrs> {
  label: Option<String>,
  ip: A,
  port: u32,
  target: u64,
  socket: Arc<UdpSocket>
}

impl<A> Bulb<A> where A : ToSocketAddrs {
  pub fn send_msg(&self, payload: Payload, ack_required: bool) -> Result<u8, String> {
    send_msg(&self.socket, &self.ip, payload, ack_required, self.target)
  } 
}

impl<A> Display for Bulb<A> where A : ToSocketAddrs + Display {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    write!(
      f, 
      "'{:?}' ({}:{} {})", 
      self.label,
      self.ip, 
      self.port,
      self.target)
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
  pub fn new<A : ToSocketAddrs>(addr: A) -> Result<Client, String> {
    let closed = Arc::new(AtomicBool::new(false));
    let devices = Arc::new(RwLock::new(HashMap::new()));
    let udp_builder = try!(
      UdpBuilder::new_v4().or(err!("failed to create builder")));
    let udp_socket = Arc::new(try!(
      udp_builder.bind(addr).or(err!("failed to bind to addr"))));

    let client = Client {
      closed: closed,
      socket: udp_socket,
      devices: devices
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
        let (amt, src) = socket.recv_from(&mut buf[..]).unwrap();
        let resp = serialize::decode::<Message>(&buf[..amt]).unwrap();
        
        match *resp.payload() {
          Payload::Device(Device::StateService(Service::Udp, port)) =>
            {
              println!("Received device with port: {}", port);

              devices.write().unwrap().entry(resp.target()).or_insert(
                Bulb { 
                  label: None, 
                  ip: src,
                  port: port, 
                  target: resp.target(),
                  socket: socket.clone()
                });

              println!("Devices:");

              for d in devices.read().unwrap().values() {
                println!("  Devices: {}", d); 

                let _ = d.send_msg(Payload::Device(Device::GetLabel), false);

                thread::sleep_ms(200);
              }
            }
          Payload::Device(Device::StateLabel(ref label)) =>
            {
              println!("Received device label: '{:?}' for {}", label, resp.target());

              // TODO: Move the label with some UNSAFE code?
              if let Some(bulb) = devices.write().unwrap().get_mut(&resp.target()) {
                bulb.label = Some(label.clone());
              }
            }
          _ => 
            ()
        }
      }
    })
  }

  /// broadcasts messages to the client at a set interval. use `listen` to 
  /// have the client process certain messages.
  ///
  pub fn discover(&self, wait: u32) -> JoinHandle<()> {
    use Device::*;

    let socket = self.socket.clone();
    let closed = self.closed.clone();

    thread::spawn(move || {
      while !closed.load(Ordering::SeqCst) {
        // TODO: Technically, there should be a LOCK on the socket here. Messages
        // should not be able to be sent between the time the socket is set to 
        // broadcast.
        let _ = socket.set_broadcast(true);
        let _ = send_msg(&socket, BROADCAST_IP, Payload::Device(GetService), false, 0);
        let _ = socket.set_broadcast(false);

        thread::sleep_ms(wait);
      }
    })
  }

  /// sends a message to the specified address.
  ///
  pub fn send_msg<A : ToSocketAddrs>(
    &self, 
    addr: A,
    payload: Payload, 
    ack_required: bool, 
    target: u64
  ) -> Result<u8, String> 
  {
    send_msg(&self.socket, addr, payload, ack_required, target)
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
  for _ in 1..(u8::MAX as usize) + 1 { next_sequence(); }
  assert_eq!(0, next_sequence());
  for _ in 1..(u8::MAX as usize) + 1 { next_sequence(); }
  assert_eq!(0, next_sequence());
}