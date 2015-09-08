#![feature(unicode)]

extern crate net2;
extern crate byteorder;
extern crate rustc_serialize;

macro_rules! err(
  ($s : expr) => (Err($s.to_string()))
);

mod client;
mod header;
mod payload;
mod message;
pub mod serialize;

pub use client::Client;
pub use header::Header;
pub use message::Message;
pub use payload::{Service, Device, Light, Payload, Power};