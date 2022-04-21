use std::fmt::Display;

use anyhow::Result;

use crate::chunk_type::ChunkType;

pub struct Chunk {
    /// The number of bytes in the chunk's data field
    length: usize,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    // cyclic redundancy check calculated on the preceding bytes in the chunk, including the chunk type code and chunk data.
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let chunk_type_vec = chunk_type
            .bytes()
            .into_iter()
            .map(|b| b)
            .collect::<Vec<u8>>();

        let size = data.len();

        let vec_to_check = [chunk_type.bytes().to_vec(), data.to_owned()].concat();
        let check = crc::crc32::checksum_ieee(&vec_to_check);
        Self {
            chunk_type,
            chunk_data: data,
            length: size,
            crc: check
        }
    }
    pub fn length(&self) -> u32 {
        self.length as u32
    }
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    pub fn data(&self) -> &[u8] {
        &self.chunk_data
    }
    pub fn crc(&self) -> u32 {
        self.crc
    }
    pub fn data_as_string(&self) -> Result<String> {
        let data = &self.chunk_data;
        let as_string = std::str::from_utf8(data)
            .map_err(anyhow::Error::msg)
            ?.to_owned();
        Ok(as_string)
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        todo!();
    }
}

impl TryFrom<&Vec<u8>> for Chunk {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let mut byte_chunk_type : [u8; 4] = Default::default();
        let value_slice = value.as_slice();

        let mut data_length_slice : [u8; 4] = Default::default();
        data_length_slice.copy_from_slice(&value[..4]);
        let data_length = bytes_to_number(&data_length_slice);
        
        byte_chunk_type.copy_from_slice(&value_slice[4..8]);

        let chunk_type = ChunkType::try_from(byte_chunk_type);
        if chunk_type.is_err() {
            return Err("Failed to build chunk type");
        }
        let rest_bytes = &value_slice[8..];

        let crc_starting_index = rest_bytes.len() - 4;
        let crc_bytes = rest_bytes[crc_starting_index..].to_vec();
        let mut crc_slice : [u8; 4] = Default::default();
        crc_slice.copy_from_slice(&crc_bytes);

        let rest_bytes = &rest_bytes[..crc_starting_index];

        let chunk = Self::new(chunk_type.unwrap(), rest_bytes.to_vec());

        if chunk.length() != data_length {
            return Err("Invalid chunk length declared");
        }
        if chunk.crc() != bytes_to_number(&crc_slice) {
            return Err("Invalid chunk CRC length declared");
        }
        Ok(chunk)
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.data_as_string().unwrap_or("Invalid chunk".to_owned())))
    }
}

fn bytes_to_number (&bytes: &[u8; 4]) -> u32 {
    ((bytes[0] as u32) << 24) +
    ((bytes[1] as u32) << 16) +
    ((bytes[2] as u32) <<  8) +
    ((bytes[3] as u32) <<  0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
        
        let _chunk_string = format!("{}", chunk);
    }
}
