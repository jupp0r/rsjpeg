#[macro_use]
extern crate nom;

#[macro_use]
extern crate failure;

mod parser;
mod errors;

pub use parser::decode;

