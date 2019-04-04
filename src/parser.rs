use nom::{be_u16, be_u8, Needed};

use failure::Error;

use errors::ParserError;

#[derive(Debug, Eq, PartialEq)]
pub enum Marker<'a> {
    Other(SomeMarker<'a>),
    DHT(DefineHuffmanTable<'a>),
    Image(ImageStream<'a>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct SomeMarker<'a> {
    pub tag: u8,
    pub length: u16,
    pub data: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub enum DHTType {
    LuminanceDC,
    LuminanceAC,
    ChrominanceDC,
    ChrominanceAC,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DefineHuffmanTable<'a> {
    pub class: DHTType,
    pub symbols: [&'a [u8]; 16],
}

#[derive(Debug, Eq, PartialEq)]
pub struct ImageStream<'a> {
    pub metadata: &'a [u8],
    pub data: &'a [u8],
}

named!(
    start_of_stream<&[u8], Marker>,
    do_parse!(
        tag!(b"\xff\xda")
            >> length: be_u16
            >> metadata: take!(length - 2)
            >> data: take_until!(&b"\xff\xd9"[..])
            >> (Marker::Image(ImageStream{ metadata, data }))
    )
);

named!(huffman_table<&[u8], Marker>,
    do_parse!(
        tag!(b"\xff\xc4")
        >> take!(2)
        >> id_class: bits!(pair!(take_bits!(u8, 4), take_bits!(u8, 4)))
        >> symbols_length: count_fixed!(u8, be_u8, 16)
        >> s1: take!(symbols_length[0])
        >> s2: take!(symbols_length[1])
        >> s3: take!(symbols_length[2]) 
        >> s4: take!(symbols_length[3])
        >> s5: take!(symbols_length[4])
        >> s6: take!(symbols_length[5])
        >> s7: take!(symbols_length[6])
        >> s8: take!(symbols_length[7])
        >> s9: take!(symbols_length[8])
        >> s10: take!(symbols_length[9])
        >> s11: take!(symbols_length[10])
        >> s12: take!(symbols_length[11])
        >> s13: take!(symbols_length[12])
        >> s14: take!(symbols_length[13])
        >> s15: take!(symbols_length[14])
        >> s16: take!(symbols_length[15])
        >> (Marker::DHT(DefineHuffmanTable{
            class: match id_class {
                (0,0) => DHTType::LuminanceDC,
                (0,1) => DHTType::LuminanceAC,
                (1,0) => DHTType::ChrominanceDC,
                (1,1) => DHTType::ChrominanceAC,
                _ => return Err(nom::Err::Incomplete(Needed::Size(5)))
            },
            symbols: [s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15, s16],
        }))
    )
);

named!(some_marker<&[u8], Marker>,
    do_parse!(
        tag!(b"\xff")
        >> tag: be_u8
        >> length: be_u16
        >> data: take!(length - 2)
        >> (Marker::Other(SomeMarker{tag, length: length - 2, data}))
    )
);



named!(jpeg<&[u8], (Vec<Marker>, &[u8])>, preceded!(soi, jfif));
named!(soi, tag!(b"\xff\xd8"));

named!(jfif<&[u8], (Vec<Marker>, &[u8])>,
      many_till!(alt!(
            complete!(start_of_stream)
          | complete!(huffman_table)
          | complete!(some_marker))
          , tag!(b"\xff\xd9")));

pub fn decode(jpeg_file: &[u8]) -> Result<Vec<Marker>, Error> {
    jpeg(jpeg_file)
        .map(|parsed_correctly| (parsed_correctly.1).0)
        .map_err(|e| {
            ParserError {
                reason: format!("{:?}", e),
            }
            .into()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decode_test() {
        let minimal_jpeg = vec![
            0xFF, 0xD8, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x03, 0x02, 0x02, 0x02, 0x02, 0x02, 0x03,
            0x02, 0x02, 0x02, 0x03, 0x03, 0x03, 0x03, 0x04, 0x06, 0x04, 0x04, 0x04, 0x04, 0x04,
            0x08, 0x06, 0x06, 0x05, 0x06, 0x09, 0x08, 0x0A, 0x0A, 0x09, 0x08, 0x09, 0x09, 0x0A,
            0x0C, 0x0F, 0x0C, 0x0A, 0x0B, 0x0E, 0x0B, 0x09, 0x09, 0x0D, 0x11, 0x0D, 0x0E, 0x0F,
            0x10, 0x10, 0x11, 0x10, 0x0A, 0x0C, 0x12, 0x13, 0x12, 0x10, 0x13, 0x0F, 0x10, 0x10,
            0x10, 0xFF, 0xC9, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00,
            0xFF, 0xCC, 0x00, 0x06, 0x00, 0x10, 0x10, 0x05, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01,
            0x00, 0x00, 0x3F, 0x00, 0xD2, 0xCF, 0x20, 0xFF, 0xD9,
        ];

        assert_eq!(
            decode(&minimal_jpeg[..]).unwrap(),
            vec![
                Marker::Other(SomeMarker {
                    tag: 0xdb,
                    length: 0x41,
                    data: &[
                        0x00, 0x03, 0x02, 0x02, 0x02, 0x02, 0x02, 0x03, 0x02, 0x02, 0x02, 0x03,
                        0x03, 0x03, 0x03, 0x04, 0x06, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x06,
                        0x06, 0x05, 0x06, 0x09, 0x08, 0x0A, 0x0A, 0x09, 0x08, 0x09, 0x09, 0x0A,
                        0x0C, 0x0F, 0x0C, 0x0A, 0x0B, 0x0E, 0x0B, 0x09, 0x09, 0x0D, 0x11, 0x0D,
                        0x0E, 0x0F, 0x10, 0x10, 0x11, 0x10, 0x0A, 0x0C, 0x12, 0x13, 0x12, 0x10,
                        0x13, 0x0F, 0x10, 0x10, 0x10
                    ]
                }),
                Marker::Other(SomeMarker {
                    tag: 0xc9,
                    length: 0x9,
                    data: &[0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00]
                }),
                Marker::Other(SomeMarker {
                    tag: 0xcc,
                    length: 0x4,
                    data: &[0x00, 0x10, 0x10, 0x05]
                }),
                Marker::Image(ImageStream {
                    metadata: &[0x01, 0x01, 0x00, 0x00, 0x3F, 0x00],
                    data: &[0xD2, 0xCF, 0x20]
                })
            ]
        );
    }
}
