#![feature(thread_sleep)]

extern crate lifx;
extern crate env_logger;

use std::thread;
use std::time::Duration;

use lifx::{GET_ALL, Client};


fn main() {
  env_logger::init().unwrap();

  let client = Client::new("0.0.0.0:1234").unwrap();
  let thread = client.listen();
  let discover_thread = client.discover(1000, GET_ALL);

  println!("Waiting 10 seconds to discover devices...");

  thread::sleep(Duration::from_secs(10));

  let devices = {
    let devices = client.devices(); 

    if devices.len() == 0 {
      println!("Waiting 15 more seconds to discover devices..."); 
      println!("  This might be caused by a busy network...");
      
      thread::sleep(Duration::from_secs(15));

      client.devices()
    } else {
      devices
    }
  };

  println!("Complete! Closing connection...");
  println!("Found: {} devices", devices.len());

  client.close();

  let _ = discover_thread.join();
  let _ = thread.join();

  for d in devices.values() {
    println!("Device: {}", d);
  }
}