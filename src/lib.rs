#[macro_use]
extern crate nom;

#[macro_use]
extern crate failure;

mod errors;
mod parser;

pub use parser::decode;
