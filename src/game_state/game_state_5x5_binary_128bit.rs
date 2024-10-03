use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, SantoriniEval, SantoriniState5x5};
use crate::game_state::utils::static_evaluation::gs5x5_static_evaluation;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;

/*
Bits 0-74: 3 bits per tile, 25 tiles
Bits 75-79: Player A worker 1 position
Bits 80-84: Player A worker 2 position
Bits 85-89: Player B worker 1 position
Bits 90-94: Player B worker 2 position
Bit  95: Player A's turn (bool)
Bits 96-125: Unused
Bit 126: Player B has won (because they have reached height 3)
Bit 127: Player A has won (because they have reached height 3)

For each tile:
- Bits 0-2: Height (0-4)

If a worker is not placed, the position is set to 0x1F (11111 in binary), which is out of bounds for a 5x5 board.
If only one worker is placed, it must be in the worker 1 position.
 */
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct GameState5x5Binary128bit(u128);

impl fmt::Display for GameState5x5Binary128bit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState5x5Binary128bit {
    const WORKER_NOT_PLACED: u8 = 25;

    const NO_NEIGHBOR: usize = usize::MAX;
    const TILE_TO_NEIGHBORS: [[usize; 8]; 25] = Self::precompute_tile_to_neighbors();
    const fn precompute_tile_to_neighbors() -> [[usize; 8]; 25] {
        let mut tile_to_neighbors = [[Self::NO_NEIGHBOR; 8]; 25];

        let mut row: isize = 0;
        while row < 5 {
            let mut column = 0;
            while column < 5 {
                let tile_id = (row * 5 + column) as usize;
                let mut tile_neighbor_index = 0;

                let mut neighbor_row = row - 1;
                while neighbor_row <= row + 1 {
                    if neighbor_row < 0 || neighbor_row >= 5 {
                        neighbor_row += 1;
                        continue;
                    }
                    let mut neighbor_column = column - 1;
                    while neighbor_column <= column + 1 {
                        if neighbor_column < 0 || neighbor_column >= 5 || (neighbor_row == row && neighbor_column == column) {
                            neighbor_column += 1;
                            continue;
                        }
                        let neighbor_tile_id = (neighbor_row * 5 + neighbor_column) as usize;
                        tile_to_neighbors[tile_id][tile_neighbor_index] = neighbor_tile_id;
                        tile_neighbor_index += 1;

                        neighbor_column += 1;
                    }
                    neighbor_row += 1;
                }
                column += 1;
            }
            row += 1;
        }
        return tile_to_neighbors;
    }

    pub fn get_tile_heights(&self) -> [u8; 25] {
        let mut tile_heights = [0; 25];

        let mut data = self.0;
        for i in 0..25 {
            tile_heights[i] = (data & 0x7) as u8;
            data >>= 3;
        }
        return tile_heights;
    }

    pub fn get_player_a_worker_tiles(&self) -> [u8; 2] {
        return [
            (self.0 >> 75 & 0x1F) as u8,
            (self.0 >> 80 & 0x1F) as u8
        ];
    }

    pub fn get_player_b_worker_tiles(&self) -> [u8; 2] {
        return [
            (self.0 >> 85 & 0x1F) as u8,
            (self.0 >> 90 & 0x1F) as u8
        ];
    }
}

impl GameState for GameState5x5Binary128bit {
    type RawValue = u128;
    type GenericGameState = GenericSantoriniGameState<5, 5, 2>;

    fn new(value: u128) -> Self {
        return GameState5x5Binary128bit(value);
    }

    fn raw_value(&self) -> u128 {
        return self.0;
    }

    fn is_player_a_turn(&self) -> bool {
        return self.0 & (1u128 << 95) != 0;
    }

    fn has_player_a_won(&self) -> bool {
        return self.0 & (1 << 127) != 0;
    }

    fn has_player_b_won(&self) -> bool {
        return self.0 & (1 << 126) != 0;
    }

    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<5, 5, 2>) -> Self {
        let mut binary_state = 0u128;

        // Set tile heights
        for tile_id in (0..25).rev() {
            let height = generic_game_state.get_tile_height(tile_id);
            binary_state <<= 3;
            binary_state |= height as u128;
        }

        // Set worker positions
        if let Some(a_workers) = generic_game_state.player_a_workers {
            for (index, worker_tile) in a_workers.iter().enumerate() {
                binary_state |= (*worker_tile as u128) << (75 + 5 * index);
            }
        } else {
            binary_state |= ((Self::WORKER_NOT_PLACED as u128) << 75) + ((Self::WORKER_NOT_PLACED as u128) << 80);
        }
        if let Some(b_workers) = generic_game_state.player_b_workers {
            for (index, worker_tile) in b_workers.iter().enumerate() {
                binary_state |= (*worker_tile as u128) << (85 + 5 * index);
            }
        } else {
            binary_state |= ((Self::WORKER_NOT_PLACED as u128) << 85) + ((Self::WORKER_NOT_PLACED as u128) << 90);
        }

        // Set player turn
        binary_state |= (generic_game_state.player_a_turn as u128) << 95;

        // Set win bits
        if generic_game_state.has_player_a_won() {
            binary_state |= 1u128 << 127;
        }
        if generic_game_state.has_player_b_won() {
            binary_state |= 1u128 << 126;
        }

        return GameState5x5Binary128bit(binary_state);
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<5, 5, 2> {
        let tile_heights = self.get_tile_heights();
        let player_a_workers = self.get_player_a_worker_tiles();
        let player_b_workers = self.get_player_b_worker_tiles();
        let player_a_turn = self.is_player_a_turn();

        let mut generic_tile_heights = [[0; 5]; 5];
        for i in 0..25 {
            generic_tile_heights[i / 5][i % 5] = tile_heights[i];
        }

        let generic_player_a_workers = if player_a_workers[0] == Self::WORKER_NOT_PLACED {
            None
        } else {
            Some(player_a_workers)
        };
        let generic_player_b_workers = if player_b_workers[0] == Self::WORKER_NOT_PLACED {
            None
        } else {
            Some(player_b_workers)
        };

        return GenericSantoriniGameState::<5, 5, 2>::new(generic_player_a_workers, generic_player_b_workers, generic_tile_heights, player_a_turn)
            .expect("Invalid game state");
    }

    fn get_children_states(&self) -> Vec<Self> {
        let mut possible_next_states = Vec::with_capacity(64);
        self.get_children_states_reuse_vec(&mut possible_next_states);
        return possible_next_states;
    }

    fn get_children_states_reuse_vec(&self, possible_next_states: &mut Vec<Self>) {
        debug_assert!(!self.has_player_a_won());
        debug_assert!(!self.has_player_b_won());

        possible_next_states.clear();

        let tile_heights = self.get_tile_heights();
        let is_player_a_turn = self.is_player_a_turn();

        let moving_player_workers = if is_player_a_turn { self.get_player_a_worker_tiles() } else { self.get_player_b_worker_tiles() };
        let other_player_workers = if is_player_a_turn { self.get_player_b_worker_tiles() } else { self.get_player_a_worker_tiles() };

        let mut tile_has_worker = [false; 25];
        for worker_tile in moving_player_workers.iter().chain(other_player_workers.iter()) {
            if *worker_tile != Self::WORKER_NOT_PLACED {
                tile_has_worker[*worker_tile as usize] = true;
            }
        }


        if moving_player_workers[0] == Self::WORKER_NOT_PLACED {
            // Workers are not placed yet, so the next states are all possible worker placements

            let worker_bit_offset = if is_player_a_turn { 75 } else { 85 };
            let new_state_base = (self.0 & !(0x3FFu128 << worker_bit_offset)) ^ (1u128 << 95);

            for worker_1_tile_id in 0..25 {
                if tile_has_worker[worker_1_tile_id] {
                    continue;
                }
                for worker_2_tile_id in (worker_1_tile_id + 1)..25 {
                    if tile_has_worker[worker_2_tile_id] {
                        continue;
                    }

                    let mut new_state = new_state_base;
                    new_state |= (worker_1_tile_id as u128) << worker_bit_offset;
                    new_state |= (worker_2_tile_id as u128) << (worker_bit_offset + 5);
                    possible_next_states.push(GameState5x5Binary128bit(new_state));
                }
            }

            return;
        }


        // All workers are placed, so the next states are all possible worker moves
        for worker_index in 0..2 {
            let worker_tile = moving_player_workers[worker_index] as usize;
            let worker_height = tile_heights[worker_tile];
            let max_movement_height = match worker_height {
                0 => 1,
                1 => 2,
                2 => 3,
                _ => panic!("Can't get children for a game state that is already won")
            };

            let worker_bit_offset = if is_player_a_turn { 75 + 5 * worker_index } else { 85 + 5 * worker_index };
            // Clearing the worker position and flipping the turn0
            let new_state_base = (self.0 & !(0x1Fu128 << worker_bit_offset)) ^ (1u128 << 95);

            for movement_tile in Self::TILE_TO_NEIGHBORS[worker_tile] {
                if movement_tile == Self::NO_NEIGHBOR {
                    break;
                }
                if tile_has_worker[movement_tile] {
                    continue;
                }
                let movement_height = tile_heights[movement_tile];
                if movement_height > max_movement_height {
                    continue;
                }

                for build_tile in Self::TILE_TO_NEIGHBORS[movement_tile] {
                    if build_tile == Self::NO_NEIGHBOR {
                        break;
                    }
                    if tile_has_worker[build_tile] && build_tile != worker_tile {
                        continue;
                    }
                    let build_height = tile_heights[build_tile];
                    if build_height >= 4 {
                        continue;
                    }

                    let mut new_state = new_state_base;
                    new_state |= (movement_tile as u128) << worker_bit_offset;
                    new_state += 1 << (build_tile * 3);
                    if movement_height == 3 {
                        if is_player_a_turn {
                            new_state |= 1u128 << 127;
                        } else {
                            new_state |= 1u128 << 126;
                        }
                    }
                    possible_next_states.push(GameState5x5Binary128bit(new_state));
                }
            }
        }
    }
}

impl SantoriniEval for GameState5x5Binary128bit {
    type SantoriniState = SantoriniState5x5;

    fn get_santorini_state(&self) -> Self::SantoriniState {
        return SantoriniState5x5 {
            tile_heights: self.get_tile_heights(),
            worker_a_tiles: self.get_player_a_worker_tiles(),
            worker_b_tiles: self.get_player_b_worker_tiles(),
            player_a_turn: self.is_player_a_turn(),
        };
    }

    fn get_child_evaluation(&self) -> f32 {
        if self.has_player_a_won() {
            return f32::INFINITY;
        } else if self.has_player_b_won() {
            return f32::NEG_INFINITY;
        }

        return gs5x5_static_evaluation::get_child_evaluation(self.get_santorini_state())
    }
}