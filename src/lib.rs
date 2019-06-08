#[macro_use]
extern crate nom;

#[macro_use]
extern crate failure;

extern crate bitvec;

mod errors;
mod huffman;
mod parser;

pub use parser::decode;
