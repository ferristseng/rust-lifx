extern crate lifx;

use lifx::Client;


fn main() {
  let client = Client::new("0.0.0.0:1234").unwrap();
  let thread = client.listen();

  client.get_services().unwrap();

  thread.join();
}