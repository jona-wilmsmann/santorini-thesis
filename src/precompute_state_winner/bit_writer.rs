use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use anyhow::Result;

const BUFFER_SIZE: usize = 1024;

pub struct BitWriter {
    writer: BufWriter<File>,
    buffer: [u8; BUFFER_SIZE],
    buffer_pos: usize,
}

impl BitWriter {
    pub async fn new(file_path: String) -> Result<Self> {
        let file = File::create(file_path).await?;
        Ok(BitWriter {
            writer: BufWriter::new(file),
            buffer: [0; BUFFER_SIZE],
            buffer_pos: 0,
        })
    }

    pub async fn write_bit(&mut self, bit: bool) -> Result<()> {
        if bit {
            let byte_index = self.buffer_pos / 8;
            let bit_index = self.buffer_pos % 8;
            self.buffer[byte_index] |= 1 << bit_index;
        }
        self.buffer_pos += 1;

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