// #![feature(unicode)]

extern crate byteorder;
extern crate net2;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;

macro_rules! err(
  ($s : expr) => (Err($s.to_string()))
);

mod client;
mod header;
mod message;
mod payload;
pub mod serialize;

pub use client::{Bulb, Client, DiscoverOptions};
pub use header::Header;
pub use message::Message;
pub use payload::{Color, Device, Light, Payload, Power, Service, HSBK,
                  MAX_BRIGHTNESS};
