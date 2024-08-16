use std::fmt;
use std::fmt::Formatter;
use anyhow::{ensure, Result};
use rand::Rng;
use crate::generic_game_state::GenericGameState;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[deprecated] // In favor of GenericSantoriniGameState<4, 4, 1>
pub struct Generic4x4GameState {
    pub player_a_tile: u8,
    pub player_b_tile: u8,
    pub tile_heights: [u8; 16],
}

#[allow(deprecated)]
impl fmt::Display for Generic4x4GameState {
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

#[allow(deprecated)]
impl fmt::Debug for Generic4x4GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[allow(deprecated)]
impl Generic4x4GameState {
    pub fn new(player_a_tile: u8, player_b_tile: u8, tile_heights: [u8; 16]) -> Result<Generic4x4GameState> {
        for i in 0..16 {
            ensure!(tile_heights[i] <= 4, "Tile {} height must be less than or equal to 4", i);
        }
        ensure!(tile_heights[player_a_tile as usize] < 4, "Player A tile height must be less than 4");
        ensure!(tile_heights[player_b_tile as usize] < 4, "Player B tile height must be less than 4");
        ensure!(player_a_tile != player_b_tile, "Player A and Player B tiles must be different");

        return Ok(Generic4x4GameState {
            player_a_tile,
            player_b_tile,
            tile_heights,
        });
    }
}

#[allow(deprecated)]
impl GenericGameState for Generic4x4GameState {
    fn generate_random_state() -> Generic4x4GameState {
        let mut tile_heights = [0; 16];
        let player_a_tile;
        let mut player_b_tile ;

        let mut rng = rand::thread_rng();

        player_a_tile = rng.gen_range(0..16) as u8;

        loop {
            player_b_tile = rng.gen_range(0..16) as u8;
            if player_b_tile != player_a_tile {
                break;
            }
        }

        for i in 0..16 {
            let max_height = if i == player_a_tile as usize || i == player_b_tile as usize { 2 } else { 4 };
            tile_heights[i] = rng.gen_range(0..max_height + 1);
        }

        return Generic4x4GameState::new(player_a_tile, player_b_tile, tile_heights).expect("Randomly generated invalid game state");
    }
    fn generate_random_state_with_blocks(mut block_amount: usize) -> Generic4x4GameState {
        let mut tile_heights = [0; 16];

        let mut rng = rand::thread_rng();

        while block_amount > 0 {
            loop {
                let tile = rng.gen_range(0..16) as usize;
                if tile_heights[tile] < 4 {
                    tile_heights[tile] += 1;
                    break;
                }
            }
            block_amount -= 1;
        }

        let player_a_tile;
        let player_b_tile;

        loop {
            let rand_tile = rng.gen_range(0..16) as u8;
            if tile_heights[rand_tile as usize] <= 2 {
                player_a_tile = rand_tile;
                break;
            }
        }
        loop {
            let rand_tile = rng.gen_range(0..16) as u8;
            if tile_heights[rand_tile as usize] <= 2 && rand_tile != player_a_tile {
                player_b_tile = rand_tile;
                break;
            }
        }

        return Generic4x4GameState::new(player_a_tile, player_b_tile, tile_heights).expect("Randomly generated invalid game state");
    }
}