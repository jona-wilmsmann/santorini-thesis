use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, SantoriniEval, SantoriniState5x5};
use crate::game_state::utils::static_evaluation::gs5x5_static_evaluation;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;

/*
Heights:
Bits 0-49: 25 tiles - 2 bits per tile (Only 4 possible heights: 0-3, if the height is 4, it is still stored as 3)
Bits 50-63: 14 bits unused

Blocked Tiles:
Bits 0-24: 25 tiles - Is tile blocked (either by a worker or height 4)
Bits 25-31: 7 bits unused

Rest:
Bits 0-4: Player A Worker 1 position (0-24)
Bits 5-9: Player A Worker 2 position (0-24)
Bits 10-14: Player B Worker 1 position (0-24)
Bits 15-19: Player B Worker 2 position (0-24)

Bit 29: Player A turn
Bit 30: Player B has won
Bit 31: Player A has won

If a worker is not placed, the position is set to 25, which is out of bounds for a 5x5 board.
 */
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct GameState5x5BinaryComposite {
    pub heights: u64,
    pub blocked_tiles: u32,
    pub rest: u32,
}

impl fmt::Display for GameState5x5BinaryComposite {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState5x5BinaryComposite {
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

    fn get_heights_and_workers(&self) -> ([u8; 25], [u8; 2], [u8; 2]) {
        let mut tile_heights = [0; 25];
        let mut player_a_workers = [0, 2];
        let mut player_b_workers = [0, 2];

        let mut worker_blocked_mask = 0u32;

        let mut rest_data = self.rest;
        // Manual iteration instead of a loop for efficiency
        let worker_tile = rest_data & 0b11111;
        player_a_workers[0] = worker_tile as u8;
        worker_blocked_mask |= 1 << worker_tile;
        rest_data >>= 5;
        let worker_tile = rest_data & 0b11111;
        player_a_workers[1] = worker_tile as u8;
        worker_blocked_mask |= 1 << worker_tile;
        rest_data >>= 5;
        let worker_tile = rest_data & 0b11111;
        player_b_workers[0] = worker_tile as u8;
        worker_blocked_mask |= 1 << worker_tile;
        rest_data >>= 5;
        let worker_tile = rest_data & 0b11111;
        player_b_workers[1] = worker_tile as u8;
        worker_blocked_mask |= 1 << worker_tile;


        let mut height_4_blocked = self.blocked_tiles & !worker_blocked_mask;

        let mut heights_data = self.heights;
        for i in 0..25 {
            let height_4 = height_4_blocked & 1 != 0;
            if height_4 {
                tile_heights[i] = 4;
            } else {
                tile_heights[i] = (heights_data & 0x3) as u8;
            }

            heights_data >>= 2;
            height_4_blocked >>= 1;
        }

        return (tile_heights, player_a_workers, player_b_workers);
    }

    fn get_player_a_worker_tiles(&self) -> [u8; 2] {
        return [
            (self.rest >> 0 & 0x1F) as u8,
            (self.rest >> 5 & 0x1F) as u8
        ];
    }

    fn get_player_b_worker_tiles(&self) -> [u8; 2] {
        return [
            (self.rest >> 10 & 0x1F) as u8,
            (self.rest >> 15 & 0x1F) as u8
        ];
    }
}

impl GameState for GameState5x5BinaryComposite {
    type RawValue = (u64, u32, u32);
    type GenericGameState = GenericSantoriniGameState<5, 5, 2>;

    fn new(value: (u64, u32, u32)) -> Self {
        return Self {
            heights: value.0,
            blocked_tiles: value.1,
            rest: value.2,
        };
    }

    fn raw_value(&self) -> (u64, u32, u32) {
        return (self.heights, self.blocked_tiles, self.rest);
    }

    fn is_player_a_turn(&self) -> bool {
        return (self.rest & (1 << 29)) != 0;
    }

    fn has_player_a_won(&self) -> bool {
        return (self.rest & (1 << 31)) != 0;
    }

    fn has_player_b_won(&self) -> bool {
        return (self.rest & (1 << 30)) != 0;
    }

    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<5, 5, 2>) -> Self {
        let mut heights = 0;
        let mut blocked_tiles = 0;
        let mut rest = 0;

        // Set heights and blocked tiles
        for tile_id in 0..25 {
            let height = generic_game_state.get_tile_height(tile_id);
            if height > 3 {
                heights |= 3 << (tile_id * 2);
                blocked_tiles |= 1 << tile_id;
            } else {
                heights |= (height as u64) << (tile_id * 2);
            }
        }

        // Set worker positions and blocked tiles
        for (worker_tiles_option, bit_offset) in [(generic_game_state.player_a_workers, 0), (generic_game_state.player_b_workers, 10)].iter() {
            if let Some(worker_tiles) = worker_tiles_option {
                for (worker_index, &worker_tile) in worker_tiles.iter().enumerate() {
                    rest |= (worker_tile as u32) << ((worker_index * 5 + *bit_offset) as u32);
                    blocked_tiles |= 1 << worker_tile;
                }
            } else {
                rest |= (Self::WORKER_NOT_PLACED as u32) << (*bit_offset as u32);
                rest |= (Self::WORKER_NOT_PLACED as u32) << ((*bit_offset + 5) as u32);
            }
        }

        // Set player turn
        if generic_game_state.player_a_turn {
            rest |= 1 << 29;
        }

        // Set win bits
        if generic_game_state.has_player_a_won() {
            rest |= 1 << 31;
        }
        if generic_game_state.has_player_b_won() {
            rest |= 1 << 30;
        }

        return Self {
            heights,
            blocked_tiles,
            rest,
        };
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<5, 5, 2> {
        let (tile_heights, player_a_workers, player_b_workers) = self.get_heights_and_workers();
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

            let worker_bit_offset = if is_player_a_turn { 0 } else { 10 };
            // Clear worker positions and flip the turn
            let new_rest_base = (self.rest & !(0x3FF << worker_bit_offset)) ^ (1 << 29);

            for worker_1_tile_id in 0..25 {
                if tile_has_worker[worker_1_tile_id] {
                    continue;
                }
                for worker_2_tile_id in (worker_1_tile_id + 1)..25 {
                    if tile_has_worker[worker_2_tile_id] {
                        continue;
                    }

                    let mut new_rest = new_rest_base;
                    new_rest |= (worker_1_tile_id as u32) << worker_bit_offset;
                    new_rest |= (worker_2_tile_id as u32) << (worker_bit_offset + 5);

                    let new_blocked_tiles = self.blocked_tiles | (1 << worker_1_tile_id) | (1 << worker_2_tile_id);

                    possible_next_states.push(Self {
                        heights: self.heights,
                        blocked_tiles: new_blocked_tiles,
                        rest: new_rest,
                    });
                }
            }

            return;
        }

        // All workers are placed, so the next states are all possible worker moves
        for worker_index in 0..2 {
            let worker_tile = moving_player_workers[worker_index] as usize;
            let worker_height = self.heights >> (worker_tile * 2) & 0x3;
            let max_movement_height = match worker_height {
                0 => 1,
                1 => 2,
                2 => 3,
                _ => panic!("Can't get children for a game state that is already won")
            };

            let worker_bit_offset = if is_player_a_turn { 5 * worker_index } else { 10 + 5 * worker_index };
            // Clearing the worker position and flipping the turn
            let new_rest_base = (self.rest & !(0x1F << worker_bit_offset)) ^ (1 << 29);

            for movement_tile in Self::TILE_TO_NEIGHBORS[worker_tile] {
                if movement_tile == Self::NO_NEIGHBOR {
                    break;
                }
                if self.blocked_tiles & (1 << movement_tile) != 0 {
                    // This checks both if the tile is blocked by a worker and if the tile is blocked by a height of 4
                    continue;
                }
                let movement_height = self.heights >> (movement_tile * 2) & 0x3;
                if movement_height > max_movement_height {
                    continue;
                }

                for build_tile in Self::TILE_TO_NEIGHBORS[movement_tile] {
                    if build_tile == Self::NO_NEIGHBOR {
                        break;
                    }
                    if self.blocked_tiles & (1 << build_tile) != 0 && build_tile != worker_tile {
                        continue;
                    }
                    let build_height = self.heights >> (build_tile * 2) & 0x3;

                    let mut new_heights = self.heights;
                    let mut new_blocked_tiles = self.blocked_tiles;
                    let mut new_rest = new_rest_base;

                    // Incrementing the height of the build tile
                    if build_height < 3 {
                        new_heights += 1 << (build_tile * 2);
                    } else {
                        new_blocked_tiles |= 1 << build_tile;
                    }

                    // Setting the new worker position
                    new_rest |= (movement_tile as u32) << worker_bit_offset;

                    // Setting the new blocked tiles for the moved worker
                    new_blocked_tiles ^= 1 << worker_tile;
                    new_blocked_tiles ^= 1 << movement_tile;

                    if movement_height == 3 {
                        if is_player_a_turn {
                            new_rest |= 1 << 31;
                        } else {
                            new_rest |= 1 << 30;
                        }
                    }
                    possible_next_states.push(Self {
                        heights: new_heights,
                        blocked_tiles: new_blocked_tiles,
                        rest: new_rest,
                    });
                }
            }
        }
    }
}

impl SantoriniEval for GameState5x5BinaryComposite {
    type SantoriniState = SantoriniState5x5;

    fn get_santorini_state(&self) -> Self::SantoriniState {
        let (tile_heights, worker_a_tiles, worker_b_tiles) = self.get_heights_and_workers();
        return SantoriniState5x5 {
            tile_heights,
            worker_a_tiles,
            worker_b_tiles,
            player_a_turn: self.is_player_a_turn(),
        };
    }

    fn get_child_evaluation(&self) -> f32 {
        return gs5x5_static_evaluation::get_child_evaluation(self.get_santorini_state());
    }
}