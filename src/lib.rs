#![feature(unicode)]

extern crate net2;
extern crate byteorder;
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
mod payload;
mod message;
pub mod serialize;

pub use header::Header;
pub use message::Message;
pub use payload::{Service, Device, Light, Payload, Power, HSBK, Color,
                  MAX_BRIGHTNESS};
pub use client::{Bulb, Client, DiscoverOptions, GET_ALL, GET_LABEL, GET_WIFI, 
                 GET_LOCATION, GET_HOST_FIRMWARE, GET_GROUP, GET_POWER, 
                 GET_HOST_INFO};
