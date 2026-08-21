use compression::prelude::*;
    
pub fn compress (data: &[u8]) -> Vec<u8> {
    data.into_iter()
        .cloned()
        .encode(&mut BZip2Encoder::new(9), Action::Finish)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

pub fn decompress (data: &[u8]) -> Vec<u8> {
    data.iter()
        .cloned()
        .decode(&mut BZip2Decoder::new())
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}