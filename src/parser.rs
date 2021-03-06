use nom::{be_u16, be_u8, Needed};

use failure::Error;

use errors::ParserError;
use huffman::{DHTType, HuffmanTable};

#[derive(Debug, Eq, PartialEq)]
pub enum Marker<'a> {
    Other(SomeMarker<'a>),
    DHT(Vec<HuffmanTable>),
    DQT(QuantizationTable<'a>),
    Image(ImageStream<'a>),
}

#[derive(Debug, Eq, PartialEq)]
pub struct SomeMarker<'a> {
    pub tag: u8,
    pub length: u16,
    pub data: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct ImageStream<'a> {
    pub metadata: StartOfStreamMetaData,
    pub data: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct QuantizationTable<'a> {
    pub id: u64,
    pub data: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct StartOfStreamMetaData {
    pub precision: u64,
    pub height: u64,
    pub width: u64,
    pub component_metadata: Vec<ColorComponentMetaData>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ColorComponentMetaData {
    pub id: u64,
    pub sampling_resolution: u64,
    pub quantization_table: u64,
}

named!(
    start_of_stream<&[u8], Marker>,
    do_parse!(
        tag!(b"\xff\xda")
            >> _length: be_u16
            >> metadata: start_of_stream_metadata
            >> data: take_until!(&b"\xff\xd9"[..])
            >> (Marker::Image(ImageStream{ metadata, data }))
    )
);

named!(start_of_stream_metadata<&[u8], StartOfStreamMetaData>,
    do_parse!(
        precision: be_u8
        >> height: be_u16
        >> width: be_u16
        >> components: be_u8
        >> component_metadata: count!(color_component_metadata, components.into())
        >> (StartOfStreamMetaData{
            precision: precision.into(),
            height: height.into(),
            width: width.into(),
            component_metadata})
    )
);

named!(color_component_metadata<&[u8], ColorComponentMetaData>,
    do_parse!(
        id: be_u8
        >> sampling_resolution: be_u8
        >> quantization_table: be_u8
        >> (ColorComponentMetaData{
            id: id.into(),
            sampling_resolution: sampling_resolution.into(),
            quantization_table: quantization_table.into()})
    )
);

named!(huffman_tables<&[u8], Marker>,
        dbg_dmp!(do_parse!(
        tag!(b"\xff\xc4")
        >> take!(2)
        >> tables: dbg_dmp!(many1!(huffman_table))
        >> (Marker::DHT(tables))
        )));

named!(huffman_table<&[u8], HuffmanTable>,
        complete!(do_parse!(
           id_class: dbg_dmp!(bits!(pair!(take_bits!(u8, 4), take_bits!(u8, 4))))
        >> symbols_length: dbg_dmp!(count_fixed!(u8, be_u8, 16))
        >> s1: dbg_dmp!(count!(be_u8, symbols_length[0].into()))
        >> s2: dbg_dmp!(count!(be_u8, symbols_length[1].into()))
        >> s3: dbg_dmp!(count!(be_u8, symbols_length[2].into()))
        >> s4: dbg_dmp!(count!(be_u8, symbols_length[3].into()))
        >> s5: dbg_dmp!(count!(be_u8, symbols_length[4].into()))
        >> s6: dbg_dmp!(count!(be_u8, symbols_length[5].into()))
        >> s7: dbg_dmp!(count!(be_u8, symbols_length[6].into()))
        >> s8: dbg_dmp!(count!(be_u8, symbols_length[7].into()))
        >> s9: dbg_dmp!(count!(be_u8, symbols_length[8].into()))
        >> s10: dbg_dmp!(count!(be_u8, symbols_length[9].into()))
        >> s11: dbg_dmp!(count!(be_u8, symbols_length[10].into()))
        >> s12: dbg_dmp!(count!(be_u8, symbols_length[11].into()))
        >> s13: dbg_dmp!(count!(be_u8, symbols_length[12].into()))
        >> s14: dbg_dmp!(count!(be_u8, symbols_length[13].into()))
        >> s15: dbg_dmp!(count!(be_u8, symbols_length[14].into()))
        >> s16: dbg_dmp!(count!(be_u8, symbols_length[15].into()))
        >> (HuffmanTable{
            class: match id_class {
                (0,0) => DHTType::LuminanceDC,
                (0,1) => DHTType::LuminanceAC,
                (1,0) => DHTType::ChrominanceDC,
                (1,1) => DHTType::ChrominanceAC,
                _ => return Err(nom::Err::Incomplete(Needed::Size(5)))
            },
            symbols: [s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15, s16],
        })
    ))
);

named!(quantization_table<&[u8], Marker>,
    do_parse!(
        tag!(b"\xff\xdb")
        >> _length: be_u16
        >> id: be_u8
        >> data: take!(64)
        >> (Marker::DQT(QuantizationTable{id: id.into(), data}))
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
      many_till!(alt_complete!(
            start_of_stream
          | huffman_tables
          | quantization_table
          | some_marker)
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
    fn huffman_test() {
        let huffman_table_sample = vec![
            0xFF, 0xC4, 0x00, 0x1F, 0x00, 0x00, 0x00, 0x07, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x05, 0x03, 0x02, 0x06, 0x01, 0x00,
            0x07, 0x08, 0x09, 0x0A, 0x0B,
        ];

        assert_eq!(
            huffman_tables(&huffman_table_sample),
            Ok((
                vec![].as_slice(),
                Marker::DHT(vec![HuffmanTable {
                    class: DHTType::LuminanceDC,
                    symbols: [
                        vec![],
                        vec![],
                        vec![0x04, 0x05, 0x03, 0x02, 0x06, 0x01, 0x00],
                        vec![0x07],
                        vec![0x08],
                        vec![0x09],
                        vec![0x0A],
                        vec![0x0B],
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                        vec![],
                    ],
                },])
            ))
        )
    }

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
                Marker::DQT(QuantizationTable {
                    id: 0,
                    data: &[
                        0x03, 0x02, 0x02, 0x02, 0x02, 0x02, 0x03, 0x02, 0x02, 0x02, 0x03, 0x03,
                        0x03, 0x03, 0x04, 0x06, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x06, 0x06,
                        0x05, 0x06, 0x09, 0x08, 0x0A, 0x0A, 0x09, 0x08, 0x09, 0x09, 0x0A, 0x0C,
                        0x0F, 0x0C, 0x0A, 0x0B, 0x0E, 0x0B, 0x09, 0x09, 0x0D, 0x11, 0x0D, 0x0E,
                        0x0F, 0x10, 0x10, 0x11, 0x10, 0x0A, 0x0C, 0x12, 0x13, 0x12, 0x10, 0x13,
                        0x0F, 0x10, 0x10, 0x10
                    ],
                }),
                Marker::Other(SomeMarker {
                    tag: 0xc9,
                    length: 0x9,
                    data: &[0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00],
                }),
                Marker::Other(SomeMarker {
                    tag: 0xcc,
                    length: 0x4,
                    data: &[0x00, 0x10, 0x10, 0x05],
                }),
                Marker::Image(ImageStream {
                    metadata: StartOfStreamMetaData {
                        precision: 1,
                        height: 0x100,
                        width: 0x3F,
                        component_metadata: vec![]
                    },
                    data: &[0xD2, 0xCF, 0x20],
                })
            ]
        );
    }
}
