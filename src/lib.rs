#[macro_use]
extern crate nom;

#[macro_use]
extern crate failure;

mod decoder;
mod errors;

pub use decoder::decode;

