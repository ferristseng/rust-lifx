#![feature(thread_sleep)]

extern crate lifx;

use std::thread;
use std::time::Duration;

use lifx::Client;
use lifx::{Payload, Device, Power, Light};


const TARGET: u64 = 3732340569040;

static ADDR: &'static str = "10.0.1.2:56700";


fn main() {
  use lifx::Light::*;

  let client = Client::new("0.0.0.0:1234").unwrap();
  let thread = client.listen();

  client.get_services().unwrap();

  thread::sleep(Duration::from_millis(50));
  client.send_msg(ADDR, Payload::Light(GetPower), false, TARGET);

  thread::sleep(Duration::from_millis(50));
  client.send_msg(ADDR, Payload::Light(SetPower(Power::Standby, 500)), true, TARGET);

  thread::sleep(Duration::from_millis(1000));
}