use std::fmt;
use std::fmt::Formatter;
use anyhow::{ensure, Result};
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::generic_game_state::GenericGameState;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct GenericSantoriniGameState<const ROWS: usize, const COLUMNS: usize, const PIECES_PER_PLAYER: usize> {
    pub player_a_turn: bool,
    pub player_a_pieces: [u8; PIECES_PER_PLAYER],
    pub player_b_pieces: [u8; PIECES_PER_PLAYER],
    pub tile_heights: [[u8; COLUMNS]; ROWS],
}

impl<const ROWS: usize, const COLUMNS: usize, const PIECES_PER_PLAYER: usize> fmt::Display for GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n---Current Turn: {}---“\n", if self.player_a_turn { "A" } else { "B" })?;
        for row in (0..ROWS).rev() {
            write!(f, "---------------------\n")?;
            for column in 0..COLUMNS {
                let tile_id = row * 4 + column;
                let height = self.tile_heights[row][column];
                let character = if self.player_a_pieces.contains(&(tile_id as u8)) {
                    'A'
                } else if self.player_b_pieces.contains(&(tile_id as u8)) {
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

impl<const ROWS: usize, const COLUMNS: usize, const PIECES_PER_PLAYER: usize> fmt::Debug for GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const PIECES_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER> {
    const PIECE_NOT_PLACED: u8 = u8::MAX;

    pub fn new(player_a_pieces: [u8; PIECES_PER_PLAYER], player_b_pieces: [u8; PIECES_PER_PLAYER], tile_heights: [[u8; COLUMNS]; ROWS], player_a_turn: bool) -> Result<GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER>> {
        let player_a_piece_count = player_a_pieces.iter().filter(|&x| *x != Self::PIECE_NOT_PLACED).count();
        let player_b_piece_count = player_b_pieces.iter().filter(|&x| *x != Self::PIECE_NOT_PLACED).count();

        if player_a_piece_count == 0 && player_b_piece_count == 0 {
            // Beginning of the game, no pieces are placed
            ensure!(player_a_turn, "At the beginning of the game, player A must place the first piece");
        }

        if player_a_piece_count != PIECES_PER_PLAYER || player_b_piece_count != PIECES_PER_PLAYER {
            // Beginning of the game, not all pieces are placed
            if player_a_turn {
                ensure!(player_a_piece_count == player_b_piece_count, "During piece placement, if it is player A's turn, both players must have the same amount of pieces placed");
            } else {
                ensure!(player_a_piece_count == player_b_piece_count + 1, "During piece placement, if it is player B's turn, player A must have one more piece placed than player B");
            }

            for column in 0..COLUMNS {
                for row in 0..ROWS {
                    ensure!(tile_heights[column][row] == 0, "At the beginning of the game, before all pieces are placed, all heights must be 0");
                }
            }
        } else {
            // Normal game state, all pieces are placed
            for column in 0..COLUMNS {
                for row in 0..ROWS {
                    ensure!(tile_heights[column][row] <= 4, "Tile {},{} height must be less than or equal to 4", column, row);
                }
            }
        }


        let mut piece_tiles = Vec::with_capacity(PIECES_PER_PLAYER * 2);
        for piece_tile in player_a_pieces.iter().chain(player_b_pieces.iter()) {
            if *piece_tile == Self::PIECE_NOT_PLACED {
                continue;
            }

            ensure!(*piece_tile < (ROWS * COLUMNS) as u8, "Piece tile {} is out of bounds, must be less than {}", piece_tile, ROWS * COLUMNS);

            ensure!(tile_heights[*piece_tile as usize / COLUMNS][*piece_tile as usize % COLUMNS] < 4, "Piece tile {} must have a height of 0", piece_tile);


            ensure!(!piece_tiles.contains(piece_tile), "Piece tiles must be unique, {} is used multiple times", piece_tile);
            piece_tiles.push(*piece_tile);
        }

        return Ok(GenericSantoriniGameState {
            player_a_turn,
            player_a_pieces,
            player_b_pieces,
            tile_heights,
        });
    }

    pub fn set_player_a_turn(&mut self, player_a_turn: bool) {
        self.player_a_turn = player_a_turn;
    }
}


impl<const ROWS: usize, const COLUMNS: usize, const PIECES_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER> {
    fn get_random_piece_positions(rng: &mut ThreadRng) -> ([u8; PIECES_PER_PLAYER], [u8; PIECES_PER_PLAYER]) {
        let mut player_a_pieces = [GenericSantoriniGameState::<ROWS, COLUMNS, PIECES_PER_PLAYER>::PIECE_NOT_PLACED; PIECES_PER_PLAYER];
        let mut player_b_pieces = [GenericSantoriniGameState::<ROWS, COLUMNS, PIECES_PER_PLAYER>::PIECE_NOT_PLACED; PIECES_PER_PLAYER];

        let mut piece_tiles = Vec::with_capacity(PIECES_PER_PLAYER * 2);

        for i in 0..(PIECES_PER_PLAYER * 2) {
            let mut piece_tile;
            loop {
                piece_tile = rng.gen_range(0..(ROWS * COLUMNS)) as u8;
                if !piece_tiles.contains(&piece_tile) {
                    break;
                }
            }
            if i < PIECES_PER_PLAYER {
                player_a_pieces[i] = piece_tile;
            } else {
                player_b_pieces[i - PIECES_PER_PLAYER] = piece_tile;
            }
            piece_tiles.push(piece_tile);
        }

        return (player_a_pieces, player_b_pieces);
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const PIECES_PER_PLAYER: usize> GenericGameState for GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER> {
    fn generate_random_state() -> GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER> {
        let mut tile_heights = [[0; COLUMNS]; ROWS];

        let mut rng = rand::thread_rng();

        let (player_a_pieces, player_b_pieces) = GenericSantoriniGameState::<ROWS, COLUMNS, PIECES_PER_PLAYER>::get_random_piece_positions(&mut rng);
        let piece_tiles = player_a_pieces.iter().chain(player_b_pieces.iter()).copied().collect::<Vec<u8>>();

        let mut block_count = 0;
        for i in 0..(ROWS * COLUMNS) {
            // 2 is the maximum height for the player tiles, 4 is the maximum height for the other tiles
            let max_height = if piece_tiles.contains(&(i as u8)) { 2 } else { 4 };
            let height: usize = rng.gen_range(0..max_height + 1);
            tile_heights[i / COLUMNS][i % COLUMNS] = height as u8;
            block_count += height;
        }

        let player_a_turn = block_count % 2 == 0;

        return GenericSantoriniGameState::new(player_a_pieces, player_b_pieces, tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }

    fn generate_random_state_with_blocks(block_amount: usize) -> GenericSantoriniGameState<ROWS, COLUMNS, PIECES_PER_PLAYER> {
        let mut tile_heights = [[0; COLUMNS]; ROWS];

        let mut rng = rand::thread_rng();

        let (player_a_pieces, player_b_pieces) = GenericSantoriniGameState::<ROWS, COLUMNS, PIECES_PER_PLAYER>::get_random_piece_positions(&mut rng);

        let mut tile_max_heights = [[4; COLUMNS]; ROWS];
        for i in 0..PIECES_PER_PLAYER {
            tile_max_heights[player_a_pieces[i] as usize / COLUMNS][player_a_pieces[i] as usize % COLUMNS] = 2;
            tile_max_heights[player_b_pieces[i] as usize / COLUMNS][player_a_pieces[i] as usize % COLUMNS] = 2;
        }

        let mut current_block_count = 0;
        while current_block_count < block_amount {
            loop {
                let tile = rng.gen_range(0..(ROWS * COLUMNS));
                if tile_heights[tile / COLUMNS][tile % COLUMNS] < tile_max_heights[tile / COLUMNS][tile % COLUMNS] {
                    tile_heights[tile / COLUMNS][tile % COLUMNS] += 1;
                    break;
                }
            }
            current_block_count += 1;
        }

        let player_a_turn = block_amount % 2 == 0;

        return GenericSantoriniGameState::new(player_a_pieces, player_b_pieces, tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }
}