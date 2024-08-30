use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use anyhow::Result;
use crate::precompute_state_winner::asset_valid_bit_count::assert_valid_bit_count;

const BUFFER_SIZE: usize = 1024;

pub struct BitWriter<const BITS_PER_ENTRY: usize> {
    writer: BufWriter<File>,
    buffer: [u8; BUFFER_SIZE],
    buffer_pos: usize,
}

impl<const BITS_PER_ENTRY: usize> BitWriter<BITS_PER_ENTRY> {
    const VALID_BIT_COUNT_ASSERTION: () = assert_valid_bit_count(BITS_PER_ENTRY);

    pub async fn new(file_path: String) -> Result<Self> {
        let file = File::create(file_path).await?;
        Ok(BitWriter {
            writer: BufWriter::new(file),
            buffer: [0; BUFFER_SIZE],
            buffer_pos: 0,
        })
    }

    pub async fn write_data(&mut self, data: u8) -> Result<()> {
        debug_assert!(BITS_PER_ENTRY == 8 || data < (1 << BITS_PER_ENTRY));

        if data != 0 {
            let byte_index = self.buffer_pos / 8;
            let bit_index = self.buffer_pos % 8;
            self.buffer[byte_index] |= data << bit_index;
        }
        self.buffer_pos += BITS_PER_ENTRY;

        if self.buffer_pos == BUFFER_SIZE * 8 {
            self.flush_buffer().await?;
        }
        return Ok(());
    }

    async fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer_pos > 0 {
            let byte_count = (self.buffer_pos + 7) / 8;
            self.writer.write_all(&self.buffer[..byte_count]).await?;
            self.buffer = [0; BUFFER_SIZE];
            self.buffer_pos = 0;
        }
        return Ok(());
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.flush_buffer().await?;
        self.writer.flush().await?;
        return Ok(());
    }
}