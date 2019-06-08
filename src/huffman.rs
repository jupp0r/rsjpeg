use std::collections::HashMap;

use bitvec::prelude::*;

use errors::ParserError;

#[derive(Debug, Eq, PartialEq)]
pub enum DHTType {
    LuminanceDC,
    LuminanceAC,
    ChrominanceDC,
    ChrominanceAC,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HuffmanTable {
    pub class: DHTType,
    pub symbols: [Vec<u16>; 16],
}

impl HuffmanTable {
    pub fn huffman_decode(&self, code: &[u8]) -> Result<Vec<u8>, ParserError> {
        let translation = self.make_translation_map();

        println!("tranlation map: {:?}", translation);

        let bits: &BitSlice = code.into();
        let mut result = Vec::new();

        let mut cursor = 0usize;

        while cursor < bits.len() {
            let cursor_before = cursor;
            for len in 1usize..=self.symbols.len() {
                if cursor + len >= bits.len() {
                    break;
                }

                let mut found = false;
                let sub_slice = &bits[cursor..(cursor + len)];

                for symbol in &self.symbols[len - 1] {
                    let bit_symbol = symbol.as_bitslice();
                    let bits = &bit_symbol[(16 - len)..16];

                    println!("testing symbol {} against {}", bits, sub_slice);
                    if bits != sub_slice {
                        continue;
                    }
                    cursor = cursor + len;
                    found = true;

                    match translation.get(&BitVec::<BigEndian, u16>::from_bitslice(bit_symbol)) {
                        Some(&translated) => {
                            result.extend_from_slice(&translated.to_be_bytes());
                            println!("translated {} to {}", symbol, translated);
                        }
                        None => {
                            return Err(ParserError {
                                reason: format!("couldn't find translation for symbol {}", symbol),
                            })
                        }
                    }
                }

                if found {
                    break;
                }
            }
            if cursor == cursor_before {
                return Err(ParserError {
                    reason: "error huffman decoding stream: symbol not found at cursor".into(),
                });
            }
        }

        Ok(result.to_vec())
    }

    fn make_translation_map(&self) -> HashMap::<BitVec<BigEndian, u16>, u16> {
        let mut translation = HashMap::<BitVec<BigEndian, u16>, u16>::new();
        let mut current_code = 0u16;

        for len in 0..self.symbols.len() {
            for i in 0..self.symbols[len].len() {
                let mut current_bits = BitVec::from_element(current_code);
                translation.insert(
                    BitVec::from_bitslice(
                        &current_bits[(self.symbols.len() - (len + 1))..self.symbols.len()],
                    ),
                    self.symbols[len][i],
                );
                current_code = current_code + 1;
            }
            current_code = current_code << 1;
        }

        translation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_translation_map_test() {
        // from https://stackoverflow.com/questions/1563883/decoding-a-jpeg-huffman-block-table
        let table = HuffmanTable {
            class: DHTType::ChrominanceAC,
            symbols: [
                vec![],
                vec![0x01],
                vec![0x02, 0x11],
                vec![0x00, 0x03, 0x04, 0x21],
                vec![0x05, 0x12, 0x31],
                vec![0x06, 0x41, 0x51, 0x61],
                vec![0x13, 0x22, 0x71, 0x81, 0x91, 0xa1],
                vec![0x14, 0x32, 0xb1, 0xd1, 0xf0],
                vec![0x15, 0x23, 0x35, 0x42, 0xb2, 0xc1],
                vec![0x07, 0x16, 0x24, 0x33, 0x52, 0x72, 0x73, 0xe1],
                vec![0x25, 0x34, 0x43, 0x53, 0x62, 0x74, 0x82, 0x94, 0xa2, 0xf1],
                vec![0x26, 0x44, 0x54, 0x63, 0x64, 0x92, 0x93, 0xc2, 0xd2],
                vec![0x55, 0x56, 0x84, 0xb3],
                vec![0x45, 0x83],
                vec![0x46, 0xa3, 0xe2],
                vec![],
            ],
        };

        let translation = table.make_translation_map();

        let v1 = bitvec!(BigEndian, u16; 0, 0);
        assert_eq!(translation.get(&v1), Some(&0x01));

        let v2 = bitvec!(BigEndian, u16; 0, 1, 0);
        assert_eq!(translation.get(&v2), Some(&0x02));

        let v3 = bitvec!(BigEndian, u16; 1, 0, 1, 0);
        assert_eq!(translation.get(&v3), Some(&0x04));

        let v4 = bitvec!(BigEndian, u16; 1, 0, 1, 1);
        assert_eq!(translation.get(&v4), Some(&0x21));
    }

    #[test]
    fn huffman_decode_test() {
        let coded = vec![0b00011100, 0b10101011];
        let decoded = vec![0x01, 0x02, 0x00, 0x11, 0x04];
        let table = HuffmanTable {
            class: DHTType::ChrominanceAC,
            symbols: [
                vec![],
                vec![0x01, 0x02],
                vec![0x03],
                vec![0x11, 0x04, 0x00],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
        };

        assert_eq!(table.huffman_decode(coded.as_slice()), Ok(decoded))
    }
}
