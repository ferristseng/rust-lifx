extern crate lifx;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::thread;
use std::time::Duration;

use lifx::Light::*;
use lifx::{Client, Color, HSBK, Payload, MAX_BRIGHTNESS};

const DELAY: u32 = 500;
const TARGET: u64 = 3732340569040;

static ADDR: &'static str = "10.0.1.4:56700";


fn main() {
  env_logger::init();

  let client = Client::new("0.0.0.0:1234").unwrap();
  let thread = client.listen();



  println!("Setting to green...");

  //let _ = client.send_msg(ADDR,
  //                        Payload::Light(SetColor(Color::Green.to_hsbk(MAX_BRIGHTNESS), DELAY)),
  //                        true,
  //                        TARGET);


  println!("Setting to white...");

  let white = HSBK::new(0, 3000, !0, 2500);
  let _ = client.send_msg(ADDR,
                          Payload::Light(SetColor(white, DELAY)),
                          false,
                          TARGET);

  thread::sleep(Duration::from_secs(5));

  let _ = client.send_msg(ADDR, Payload::Light(Get), false, TARGET);

  thread::sleep(Duration::from_secs(5));

  client.close();

  let _ = thread.join();
}
