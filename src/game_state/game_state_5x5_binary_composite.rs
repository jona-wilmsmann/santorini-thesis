use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, MinimaxReady};
use crate::game_state::game_state_5x5_binary_128bit::GameState5x5Binary128bit;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use crate::minimax::minimax_cache::MinimaxCache;

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

If a worker is not placed, the position is set to 0x1F (11111 in binary), which is out of bounds for a 5x5 board.
If only one worker is placed, it must be in the worker 1 position.
 */
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
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

impl fmt::Debug for GameState5x5BinaryComposite {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState5x5BinaryComposite {
    const WORKER_NOT_PLACED: u8 = 0x1F;
    const NO_NEIGHBOR: usize = usize::MAX;
    const TILE_TO_NEIGHBORS: [[usize; 8]; 25] = Self::precompute_tile_to_neighbors();
    const fn precompute_tile_to_neighbors() -> [[usize; 8]; 25] {
        let mut position_to_neighbors = [[Self::NO_NEIGHBOR; 8]; 25];

        let mut row: isize = 0;
        while row < 5 {
            let mut column = 0;
            while column < 5 {
                let tile_id = (row * 5 + column) as usize;
                let mut position_neighbor_index = 0;

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
                        position_to_neighbors[tile_id][position_neighbor_index] = neighbor_tile_id;
                        position_neighbor_index += 1;

                        neighbor_column += 1;
                    }
                    neighbor_row += 1;
                }
                column += 1;
            }
            row += 1;
        }
        return position_to_neighbors;
    }


    pub fn get_position_heights(&self) -> [u8; 25] {
        let mut position_heights = [0; 25];

        let mut worker_on_tile = [false; 25];
        let mut rest_data = self.rest;
        for i in 0..4 {
            let worker_tile = rest_data as u8 & 0x1F;
            if worker_tile != Self::WORKER_NOT_PLACED {
                worker_on_tile[worker_tile as usize] = true;
            }
            rest_data >>= 5;
        }

        let mut heights_data = self.heights;
        let mut blocked_data = self.blocked_tiles;
        for i in 0..25 {
            let blocked = blocked_data & 1 != 0;
            let height = if blocked && !worker_on_tile[i] {
                4
            } else {
                heights_data as u8 & 0x3
            };

            position_heights[i] = height;

            heights_data >>= 2;
            blocked_data >>= 1;
        }
        return position_heights;
    }

    pub fn get_player_a_worker_tiles(&self) -> [u8; 2] {
        return [
            (self.rest >> 0 & 0x1F) as u8,
            (self.rest >> 5 & 0x1F) as u8
        ];
    }

    pub fn get_player_b_worker_tiles(&self) -> [u8; 2] {
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
        for (worker_tile, bit_offset) in [(generic_game_state.player_a_workers[0], 0), (generic_game_state.player_a_workers[1], 5), (generic_game_state.player_b_workers[0], 10), (generic_game_state.player_b_workers[1], 15)].iter() {
            if worker_tile == &GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED {
                rest |= (Self::WORKER_NOT_PLACED as u32) << (*bit_offset as u32);
            } else {
                rest |= (*worker_tile as u32) << (*bit_offset as u32);
                blocked_tiles |= 1 << (*worker_tile as u32);
            }
        }

        // Set player turn
        if generic_game_state.player_a_turn {
            rest |= 1 << 29;
        }

        // Set win bits
        if generic_game_state.player_a_workers.iter().filter(|&x| *x != GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED).any(|&x| generic_game_state.get_tile_height(x as usize) == 3) {
            rest |= 1 << 31;
        }
        if generic_game_state.player_b_workers.iter().filter(|&x| *x != GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED).any(|&x| generic_game_state.get_tile_height(x as usize) == 3) {
            rest |= 1 << 30;
        }

        return Self {
            heights,
            blocked_tiles,
            rest,
        };
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<5, 5, 2> {
        let mut tile_heights = [[0; 5]; 5];
        let mut player_a_workers = [GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED; 2];
        let mut player_b_workers = [GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED; 2];
        let player_a_turn = self.is_player_a_turn();

        let mut tile_blocked_by_worker = [false; 25];

        let mut worker_state = self.rest;
        for pos in player_a_workers.iter_mut().chain(player_b_workers.iter_mut()) {
            let worker_pos = worker_state as u8 & 0x1F;
            if worker_pos != Self::WORKER_NOT_PLACED {
                *pos = worker_pos;
                tile_blocked_by_worker[worker_pos as usize] = true;
            }
            worker_state >>= 5;
        }

        let mut heights = self.heights;
        let mut blocked_tiles = self.blocked_tiles;
        for tile_id in 0..25 {
            let blocked = blocked_tiles & 1 != 0;
            let height = if blocked && !tile_blocked_by_worker[tile_id] {
                4
            } else {
                heights as u8 & 0x3
            };

            tile_heights[tile_id / 5][tile_id % 5] = height;

            heights >>= 2;
            blocked_tiles >>= 1;
        }

        return GenericSantoriniGameState::<5, 5, 2>::new(player_a_workers, player_b_workers, tile_heights, player_a_turn)
            .expect("Invalid game state");
    }

    fn get_children_states(&self) -> Vec<Self> {
        return self.get_children_states_reuse_vec(Vec::with_capacity(64));
    }

    fn get_children_states_reuse_vec(&self, vec: Vec<Self>) -> Vec<Self> {
        debug_assert!(!self.has_player_a_won());
        debug_assert!(!self.has_player_b_won());

        let mut possible_next_states = vec;
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

        let worker_index_to_place_option = moving_player_workers.iter().position(|&x| x == Self::WORKER_NOT_PLACED);

        if let Some(worker_index_to_place) = worker_index_to_place_option {
            // Not all workers are placed yet, so the next states are all possible worker placements
            let worker_bit_offset = if is_player_a_turn { 5 * worker_index_to_place } else { 10 + 5 * worker_index_to_place };
            let new_rest_base = (self.rest & !(0x1F << worker_bit_offset)) ^ (1 << 29);

            for tile_id in 0..25 {
                if !tile_has_worker[tile_id] {
                    let mut new_rest = new_rest_base;
                    new_rest |= (tile_id as u32) << worker_bit_offset;

                    let new_blocked_tiles = self.blocked_tiles | (1 << tile_id);

                    possible_next_states.push(Self {
                        heights: self.heights,
                        blocked_tiles: new_blocked_tiles,
                        rest: new_rest,
                    });
                }
            }

            return possible_next_states;
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

        return possible_next_states;
    }

    fn get_flipped_state(&self) -> Self {
        let mut flipped_rest = 0;

        flipped_rest |= (self.rest & 0x3FF) << 10;
        flipped_rest |= (self.rest & (0x3FF << 10)) >> 10;

        flipped_rest |= !(self.rest & (1 << 29));
        if self.has_player_a_won() {
            flipped_rest |= 1 << 30;
        }
        if self.has_player_b_won() {
            flipped_rest |= 1 << 31;
        }

        return Self {
            heights: self.heights,
            blocked_tiles: self.blocked_tiles,
            rest: flipped_rest,
        };
    }
}

impl GameState5x5BinaryComposite {
    const DISTANCE_TO_STATIC_VALUATION: [f32; 5] = [5.0, 2.0, 1.0, 0.5, 0.0];
    const HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION: [[f32; 5]; 3] = [
        [1.0, 1.5, -1.0, 0.0, -1.0], //Start height 0
        [1.0, 1.5, 2.0, 0.5, -1.0], //Start height 1
        [1.0, 1.5, 2.0, 3.0, -1.0], //Start height 2
    ];

    const TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION: [[[[f32; 5]; 3]; 25]; 25] =
        Self::precompute_tile_to_tile_to_height_to_height_to_valuation();
    const fn precompute_tile_to_tile_to_height_to_height_to_valuation() -> [[[[f32; 5]; 3]; 25]; 25] {
        let mut tile_to_tile_to_height_to_height_to_valuation = [[[[0.0; 5]; 3]; 25]; 25];

        let mut i = 0;
        while i < 25 {
            let row_i = i / 5;
            let column_i = i % 5;
            let mut j = 0;
            while j < 25 {
                let row_j = j / 5;
                let column_j = j % 5;

                let row_distance = if row_i > row_j { row_i - row_j } else { row_j - row_i };
                let column_distance = if column_i > column_j { column_i - column_j } else { column_j - column_i };
                let distance = if row_distance > column_distance { row_distance } else { column_distance };

                let mut start_height = 0;
                while start_height <= 2 {
                    let mut neighbor_height = 0;
                    while neighbor_height <= 4 {
                        let height_valuation = Self::HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION[start_height][neighbor_height];
                        let distance_valuation = Self::DISTANCE_TO_STATIC_VALUATION[distance];
                        tile_to_tile_to_height_to_height_to_valuation[i][j][start_height][neighbor_height] = height_valuation * distance_valuation;
                        neighbor_height += 1;
                    }

                    start_height += 1;
                }
                j += 1;
            }
            i += 1;
        }

        return tile_to_tile_to_height_to_height_to_valuation;
    }
}

impl MinimaxReady for GameState5x5BinaryComposite {
    fn sort_children_states(children_states: &mut Vec<Self>, depth: usize, _cache: &mut MinimaxCache<Self>) {
        if depth > 2 {
            // Create a vector of tuples with the static evaluation and the GameState
            let mut children_evaluations: Vec<(Self, f32)> = children_states.iter().map(|state| (state.clone(), state.get_static_evaluation())).collect();
            // Sort the vector by the static evaluation
            children_evaluations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            // Replace the children_states vector with the sorted vector
            *children_states = children_evaluations.iter().map(|(state, _)| state.clone()).collect();
        }
    }

    fn get_static_evaluation(&self) -> f32 {
        if self.has_player_a_won() {
            return f32::MAX;
        } else if self.has_player_b_won() {
            return f32::MIN;
        }

        let player_a_workers = self.get_player_a_worker_tiles().iter().map(|&x| x as usize).collect::<Vec<usize>>();
        let player_b_workers = self.get_player_b_worker_tiles().iter().map(|&x| x as usize).collect::<Vec<usize>>();

        let tile_heights = self.get_position_heights();

        let mut valuation = 0.0;

        // TODO: Consider whose turn it is

        for i in 0..25 {
            for player_a_worker in &player_a_workers {
                valuation += Self::TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[*player_a_worker][i][tile_heights[*player_a_worker] as usize][tile_heights[i] as usize];
            }
            for player_b_worker in &player_b_workers {
                valuation -= Self::TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[*player_b_worker][i][tile_heights[*player_b_worker] as usize][tile_heights[i] as usize];
            }
        }

        return valuation;
    }
}