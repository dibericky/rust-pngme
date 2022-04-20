use std::{str::FromStr, fmt::Display};

use anyhow::Result;


#[derive(Debug, PartialEq, Eq)]
pub struct ChunkType {
    bytes: Vec<u8>,
}

fn fifth_bit_is_zero (byte: &u8) -> bool {
    let fifth_bit_selector : u8 = 0b00100000;
    let and_comparison = byte & fifth_bit_selector;
    and_comparison == 0
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        let slice = self.bytes
            .as_slice()
            .try_into()
            .expect("Invalid slice size");
        slice
    }

    fn is_bytes_value_valid (&self) -> bool {
        self.bytes
            .iter()
            .all(|byte| match byte {
                &x if x >= 65 && x <= 90 => true,
                &x if x >= 97 && x <= 122 => true,
                _ => false
            })
    }
    pub fn is_valid(&self) -> bool {
        self.is_bytes_value_valid() && self.is_reserved_bit_valid()
    }
    pub fn is_critical(&self) -> bool {
        fifth_bit_is_zero(&self.bytes[0])
    }
    pub fn is_public(&self) -> bool {
        fifth_bit_is_zero(&self.bytes[1])
    }
    pub fn is_reserved_bit_valid(&self) -> bool {
        fifth_bit_is_zero(&self.bytes[2])
    }
    pub fn is_safe_to_copy(&self) -> bool {
        !fifth_bit_is_zero(&self.bytes[3])
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = &'static str;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(Self {
            bytes: value.to_vec()
        })
    }
}

impl FromStr for ChunkType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = s.as_bytes().to_vec();
        if vec.len() > 4 {
            return Err("Invalid size");
        }
        let chunk = Self{
            bytes: vec,
        };
        if !chunk.is_bytes_value_valid() {
            return Err("Invalid chunk");
        }
       Ok(chunk)
    }
}

impl From<String> for ChunkType {
    fn from(s: String) -> Self {
        Self{
            bytes: s.as_bytes().to_vec(),
        }
     }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stringified = String::from_utf8(self.bytes.to_owned()).unwrap();
        
        f.write_fmt(format_args!("{}", stringified))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
