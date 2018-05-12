extern crate lifx;
extern crate env_logger;

use lifx::Light::*;
use lifx::{Client, Payload, Power};


const TARGET: u64 = 3732340569040;

static ADDR: &'static str = "10.0.1.4:56700";


fn main() {
  env_logger::init();

  let client = Client::new("0.0.0.0:1234").unwrap();
  let _ = client.send_msg(ADDR,
                          Payload::Light(SetPower(Power::Max, 500)),
                          true,
                          TARGET);

  client.close();
}
