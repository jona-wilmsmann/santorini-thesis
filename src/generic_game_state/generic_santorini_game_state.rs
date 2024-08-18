use std::fmt;
use std::fmt::Formatter;
use anyhow::{ensure, Result};
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::generic_game_state::GenericGameState;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct GenericSantoriniGameState<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> {
    pub player_a_turn: bool,
    pub player_a_workers: [u8; WORKERS_PER_PLAYER],
    pub player_b_workers: [u8; WORKERS_PER_PLAYER],
    pub tile_heights: [[u8; COLUMNS]; ROWS],
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> fmt::Display for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n---Current Turn: {}---“\n", if self.player_a_turn { "A" } else { "B" })?;
        for row in (0..ROWS).rev() {
            write!(f, "---------------------\n")?;
            for column in 0..COLUMNS {
                let tile_id = row * 4 + column;
                let height = self.tile_heights[row][column];
                let character = if self.player_a_workers.contains(&(tile_id as u8)) {
                    'A'
                } else if self.player_b_workers.contains(&(tile_id as u8)) {
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

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> fmt::Debug for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    const WORKER_NOT_PLACED: u8 = u8::MAX;

    pub fn new(player_a_workers: [u8; WORKERS_PER_PLAYER], player_b_workers: [u8; WORKERS_PER_PLAYER], tile_heights: [[u8; COLUMNS]; ROWS], player_a_turn: bool) -> Result<GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER>> {
        let player_a_worker_count = player_a_workers.iter().filter(|&x| *x != Self::WORKER_NOT_PLACED).count();
        let player_b_worker_count = player_b_workers.iter().filter(|&x| *x != Self::WORKER_NOT_PLACED).count();

        if player_a_worker_count == 0 && player_b_worker_count == 0 {
            // Beginning of the game, no workers are placed
            ensure!(player_a_turn, "At the beginning of the game, player A must place the first worker");
        }

        if player_a_worker_count != WORKERS_PER_PLAYER || player_b_worker_count != WORKERS_PER_PLAYER {
            // Beginning of the game, not all workers are placed
            if player_a_turn {
                ensure!(player_a_worker_count == player_b_worker_count, "During worker placement, if it is player A's turn, both players must have the same amount of workers placed");
            } else {
                ensure!(player_a_worker_count == player_b_worker_count + 1, "During worker placement, if it is player B's turn, player A must have one more worker placed than player B");
            }

            for column in 0..COLUMNS {
                for row in 0..ROWS {
                    ensure!(tile_heights[column][row] == 0, "At the beginning of the game, before all workers are placed, all heights must be 0");
                }
            }
        } else {
            // Normal game state, all workers are placed
            for column in 0..COLUMNS {
                for row in 0..ROWS {
                    ensure!(tile_heights[column][row] <= 4, "Tile {},{} height must be less than or equal to 4", column, row);
                }
            }
        }


        let mut worker_tiles = Vec::with_capacity(WORKERS_PER_PLAYER * 2);
        for worker_tile in player_a_workers.iter().chain(player_b_workers.iter()) {
            if *worker_tile == Self::WORKER_NOT_PLACED {
                continue;
            }

            ensure!(*worker_tile < (ROWS * COLUMNS) as u8, "Worker tile {} is out of bounds, must be less than {}", worker_tile, ROWS * COLUMNS);

            ensure!(tile_heights[*worker_tile as usize / COLUMNS][*worker_tile as usize % COLUMNS] < 4, "Worker tile {} must have a height of 0", worker_tile);


            ensure!(!worker_tiles.contains(worker_tile), "Worker tiles must be unique, {} is used multiple times", worker_tile);
            worker_tiles.push(*worker_tile);
        }

        return Ok(GenericSantoriniGameState {
            player_a_turn,
            player_a_workers,
            player_b_workers,
            tile_heights,
        });
    }

    pub fn set_player_a_turn(&mut self, player_a_turn: bool) {
        self.player_a_turn = player_a_turn;
    }
}


impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn get_random_worker_positions(rng: &mut ThreadRng) -> ([u8; WORKERS_PER_PLAYER], [u8; WORKERS_PER_PLAYER]) {
        let mut player_a_workers = [GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::WORKER_NOT_PLACED; WORKERS_PER_PLAYER];
        let mut player_b_workers = [GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::WORKER_NOT_PLACED; WORKERS_PER_PLAYER];

        let mut worker_tiles = Vec::with_capacity(WORKERS_PER_PLAYER * 2);

        for i in 0..(WORKERS_PER_PLAYER * 2) {
            let mut worker_tile;
            loop {
                worker_tile = rng.gen_range(0..(ROWS * COLUMNS)) as u8;
                if !worker_tiles.contains(&worker_tile) {
                    break;
                }
            }
            if i < WORKERS_PER_PLAYER {
                player_a_workers[i] = worker_tile;
            } else {
                player_b_workers[i - WORKERS_PER_PLAYER] = worker_tile;
            }
            worker_tiles.push(worker_tile);
        }

        return (player_a_workers, player_b_workers);
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericGameState for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn generate_random_state() -> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
        let mut tile_heights = [[0; COLUMNS]; ROWS];

        let mut rng = rand::thread_rng();

        let (player_a_workers, player_b_workers) = GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::get_random_worker_positions(&mut rng);
        let worker_tiles = player_a_workers.iter().chain(player_b_workers.iter()).copied().collect::<Vec<u8>>();

        let mut block_count = 0;
        for i in 0..(ROWS * COLUMNS) {
            // 2 is the maximum height for the player tiles, 4 is the maximum height for the other tiles
            let max_height = if worker_tiles.contains(&(i as u8)) { 2 } else { 4 };
            let height: usize = rng.gen_range(0..max_height + 1);
            tile_heights[i / COLUMNS][i % COLUMNS] = height as u8;
            block_count += height;
        }

        let player_a_turn = block_count % 2 == 0;

        return GenericSantoriniGameState::new(player_a_workers, player_b_workers, tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }

    fn generate_random_state_with_blocks(block_amount: usize) -> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
        let mut tile_heights = [[0; COLUMNS]; ROWS];

        let mut rng = rand::thread_rng();

        let (player_a_workers, player_b_workers) = GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::get_random_worker_positions(&mut rng);

        let mut tile_max_heights = [[4; COLUMNS]; ROWS];
        for i in 0..WORKERS_PER_PLAYER {
            tile_max_heights[player_a_workers[i] as usize / COLUMNS][player_a_workers[i] as usize % COLUMNS] = 2;
            tile_max_heights[player_b_workers[i] as usize / COLUMNS][player_a_workers[i] as usize % COLUMNS] = 2;
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

        return GenericSantoriniGameState::new(player_a_workers, player_b_workers, tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }
}