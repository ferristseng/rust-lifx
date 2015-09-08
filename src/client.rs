use std::ops::Drop;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{Ordering, AtomicBool};
use std::collections::HashMap;
use std::net::{UdpSocket, ToSocketAddrs};

use serialize;
use message::Message;
use payload::{Payload, Device};
use net2::{UdpBuilder, UdpSocketExt};


static BROADCAST_IP: &'static str = "255.255.255.255:56700";


#[derive(Default)]
pub struct Bulb {
  label: String,
  port: u32,
  target: u64
}


/// the client handles device messages from from any lifx bulb. 
///
pub struct Client {
  closed: Arc<AtomicBool>,
  socket: Arc<UdpSocket>,
  devices: Arc<RwLock<HashMap<u64, Bulb>>>,
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

  pub fn listen(&self) -> JoinHandle<()> {
    let socket = self.socket.clone();
    let closed = self.closed.clone();

    thread::spawn(move || {
      let mut buf = [0; 256];

      while !closed.load(Ordering::SeqCst) {
        let (amt, src) = socket.recv_from(&mut buf[..]).unwrap();
        let resp = serialize::decode::<Message>(&buf[..amt]).unwrap();
        
        println!("{:?}", resp);
      }
    })
  }

  /// broadcasts a Device::GetService payload.
  ///
  pub fn get_services(&self) -> Result<(), String> {
    use Device::*;

    try!(self.socket.set_broadcast(true).or(err!("failed to turn on to broadcast")));
    try!(self.send_msg(BROADCAST_IP, Payload::Device(GetService), false, 0));
    try!(self.socket.set_broadcast(false).or(err!("failed to turn off broadcast")));
    Ok(())
  }

  /// sends a message to the specified address.
  ///
  pub fn send_msg<A : ToSocketAddrs>(
    &self, 
    addr: A,
    payload: Payload, 
    ack_required: bool, 
    target: u64
  ) -> Result<(), String> 
  {
    let msg = Message::new(payload, ack_required, target);
    let encoded = try!(serialize::encode(&msg).or(err!("failed to encode")));
    let bytes = try!(
      self.socket.send_to(&encoded[..], addr).or(err!("failed to send message")));

    if bytes == encoded.len() {
      Ok(())
    } else {
      err!("wrong number of bytes written")
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