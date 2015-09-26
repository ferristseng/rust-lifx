extern crate lifx;
extern crate env_logger;

use lifx::Client;


fn main() {
  env_logger::init().unwrap();

  let client = Client::new("0.0.0.0:1234").unwrap();
  let thread = client.listen();
  let discover_thread = client.discover(1000);

  thread.join();
}