use tokio::fs::File;
use anyhow::Result;
use tokio::io::AsyncReadExt;

pub struct BitVector {
    data: Vec<u8>,
}

impl BitVector {
    pub async fn from_file(filename: &str) -> Result<Self> {
        let mut file = File::open(filename).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;
        Ok(BitVector {
            data
        })
    }

    pub fn new_empty() -> Self {
        BitVector {
            data: Vec::new()
        }
    }

    pub fn get(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = index % 8;
        return (self.data[byte_index] & (1 << bit_index)) != 0
    }
}