extern crate lifx;

use lifx::Client;


fn main() {
  let client = Client::new("0.0.0.0:1234").unwrap();
  let thread = client.listen();
  let discover_thread = client.discover(1000);

  thread.join();
}