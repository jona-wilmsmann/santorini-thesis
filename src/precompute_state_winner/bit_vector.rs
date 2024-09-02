use tokio::fs::File;
use anyhow::Result;
use tokio::io::AsyncReadExt;

use crate::precompute_state_winner::asset_valid_bit_count::assert_valid_bit_count;

pub struct BitVector<const BITS_PER_ENTRY: usize> {
    data: Vec<u8>,
}

impl<const BITS_PER_ENTRY: usize> BitVector<BITS_PER_ENTRY> {
    const VALID_BIT_COUNT_ASSERTION: () = assert_valid_bit_count(BITS_PER_ENTRY);
    const CHUNKS_PER_BYTE: usize = 8 / BITS_PER_ENTRY;
    const BITMASK: u8 = (1u8 << BITS_PER_ENTRY).wrapping_sub(1);

    pub async fn from_file(filename: &str) -> Result<Self> {
        return Self::from_file_with_expected_length(filename, 0).await;
    }

    pub async fn from_file_with_expected_length(filename: &str, expected_length_bytes: usize) -> Result<Self> {
        let mut file = File::open(filename).await?;
        let mut data = Vec::with_capacity(expected_length_bytes);
        file.read_to_end(&mut data).await?;
        return Ok(BitVector {
            data
        });
    }

    pub fn new_empty() -> Self {
        return BitVector {
            data: Vec::new()
        };
    }

    pub fn get(&self, index: usize) -> u8 {
        let byte_index = index / Self::CHUNKS_PER_BYTE;
        let bit_index = (index * BITS_PER_ENTRY) % 8;
        let byte = self.data[byte_index];
        return (byte >> bit_index) & Self::BITMASK;
    }
}