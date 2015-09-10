extern crate lifx;
extern crate rustc_serialize;

use lifx::Client;
use lifx::{Payload, Light, HSBK, Service, Device};
use lifx::serialize;

use rustc_serialize::json;


fn main() {
  let payload = Payload::Light(Light::State(HSBK::new(1, 2, 3, 2501), 1, "Test".to_string()));
  let payload2 = Payload::Device(Device::StateService(Service::Udp, 5000));
  let json = json::as_pretty_json(&payload);
  let json2 = json::as_pretty_json(&payload2);
  println!("{}", json);
  println!("{}", json2);
  let encoded = serialize::encode(&payload2).unwrap();
  for b in encoded.iter() {
    println!("{:#X}", b);
  }

  //let client = Client::new("0.0.0.0:1234").unwrap();
  //let thread = client.listen();

  //client.get_services().unwrap();

  //thread.join();
}