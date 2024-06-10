use std::fmt;
use std::fmt::Formatter;
use anyhow::{ensure, Result};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct GenericGameState {
    pub player_a_tile: u8,
    pub player_b_tile: u8,
    pub tile_heights: [u8; 16],
}

impl fmt::Display for GenericGameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in (0..4).rev() {
            write!(f, "---------------------\n")?;
            for column in 0..4 {
                let tile_id = row * 4 + column;
                let height = self.tile_heights[tile_id];
                let character = if self.player_a_tile as usize == tile_id {
                    'A'
                } else if self.player_b_tile as usize == tile_id {
                    'B'
                } else {
                    ' '
                };
                write!(f, "| {}{} ", height, character)?;
            }
            write!(f, "|\n")?;
        }
        write!(f, "---------------------")?;
        Ok(())
    }
}

impl fmt::Debug for GenericGameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl GenericGameState {
    pub fn new(player_a_tile: u8, player_b_tile: u8, tile_heights: [u8; 16]) -> Result<GenericGameState> {
        for i in 0..16 {
            ensure!(tile_heights[i] <= 4, "Tile {} height must be less than or equal to 4", i);
        }
        ensure!(tile_heights[player_a_tile as usize] < 4, "Player A tile height must be less than 4");
        ensure!(tile_heights[player_b_tile as usize] < 4, "Player B tile height must be less than 4");
        ensure!(player_a_tile != player_b_tile, "Player A and Player B tiles must be different");

        return Ok(GenericGameState {
            player_a_tile,
            player_b_tile,
            tile_heights,
        });
    }
}