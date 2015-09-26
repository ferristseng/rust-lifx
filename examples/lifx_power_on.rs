extern crate lifx;
extern crate env_logger;

use lifx::Client;
use lifx::{Payload, Power};


const TARGET: u64 = 3732340569040;

static ADDR: &'static str = "10.0.1.2:56700";


fn main() {
  use lifx::Light::*;

  env_logger::init().unwrap();


  let client = Client::new("0.0.0.0:1234").unwrap();
  let _ = client.send_msg(
    ADDR, 
    Payload::Light(SetPower(Power::Standby, 500)), 
    true, 
    TARGET);

  client.close();
}