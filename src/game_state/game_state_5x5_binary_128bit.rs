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
use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, MinimaxReady};
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use crate::minimax::minimax_cache::MinimaxCache;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct GameState5x5Binary128bit(u128);

impl fmt::Display for GameState5x5Binary128bit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState5x5Binary128bit {
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

        let mut data = self.0;
        for i in 0..25 {
            position_heights[i] = (data & 0x7) as u8;
            data >>= 3;
        }
        return position_heights;
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
        for (worker_tile, bit_offset) in [(generic_game_state.player_a_workers[0], 75), (generic_game_state.player_a_workers[1], 80), (generic_game_state.player_b_workers[0], 85), (generic_game_state.player_b_workers[1], 90)].iter() {
            if worker_tile == &GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED {
                binary_state |= (GameState5x5Binary128bit::WORKER_NOT_PLACED as u128) << *bit_offset;
            } else {
                binary_state |= (*worker_tile as u128) << *bit_offset;
            }
        }

        // Set player turn
        binary_state |= (generic_game_state.player_a_turn as u128) << 95;

        // Set win bits
        if generic_game_state.player_a_workers.iter().filter(|&x| *x != GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED).any(|&x| generic_game_state.get_tile_height(x as usize) == 3) {
            binary_state |= 1u128 << 127;
        }
        if generic_game_state.player_b_workers.iter().filter(|&x| *x != GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED).any(|&x| generic_game_state.get_tile_height(x as usize) == 3) {
            binary_state |= 1u128 << 126;
        }

        return GameState5x5Binary128bit(binary_state);
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<5, 5, 2> {
        let mut tile_heights = [[0; 5]; 5];
        let mut player_a_workers = [GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED; 2];
        let mut player_b_workers = [GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED; 2];
        let player_a_turn = self.0 & (1u128 << 95) != 0;

        let mut state = self.0;
        for tile_id in 0..25 {
            tile_heights[tile_id / 5][tile_id % 5] = state as u8 & 0x7;
            state >>= 3;
        }

        for pos in player_a_workers.iter_mut().chain(player_b_workers.iter_mut()) {
            let worker_pos = state as u8 & 0x1F;
            if worker_pos != GameState5x5Binary128bit::WORKER_NOT_PLACED {
                *pos = worker_pos;
            }
            state >>= 5;
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

        let position_heights = self.get_position_heights();
        let is_player_a_turn = self.is_player_a_turn();

        let moving_player_workers = if is_player_a_turn { self.get_player_a_worker_tiles() } else { self.get_player_b_worker_tiles() };
        let other_player_workers = if is_player_a_turn { self.get_player_b_worker_tiles() } else { self.get_player_a_worker_tiles() };

        let mut tile_has_worker = [false; 25];
        for worker_tile in moving_player_workers.iter().chain(other_player_workers.iter()) {
            if *worker_tile != Self::WORKER_NOT_PLACED {
                tile_has_worker[*worker_tile as usize] = true;
            }
        }

        let worker_index_to_place_option = moving_player_workers.iter().position(|&x| x == GameState5x5Binary128bit::WORKER_NOT_PLACED);

        if let Some(worker_index_to_place) = worker_index_to_place_option {
            // Not all workers are placed yet, so the next states are all possible worker placements
            let worker_bit_offset = if is_player_a_turn { 75 + 5 * worker_index_to_place } else { 85 + 5 * worker_index_to_place };
            let new_state_base = (self.0 & !(0x1Fu128 << worker_bit_offset)) ^ (1u128 << 95);

            for tile_id in 0..25 {
                if !tile_has_worker[tile_id] {
                    let mut new_state = new_state_base;
                    new_state |= (tile_id as u128) << worker_bit_offset;
                    possible_next_states.push(GameState5x5Binary128bit(new_state));
                }
            }

            return possible_next_states;
        }

        // All workers are placed, so the next states are all possible worker moves
        for worker_index in 0..2 {
            let worker_tile = moving_player_workers[worker_index] as usize;
            let worker_height = position_heights[worker_tile];
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
                let movement_height = position_heights[movement_tile];
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
                    let build_height = position_heights[build_tile];
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

        return possible_next_states;
    }

    fn get_flipped_state(&self) -> Self {
        let mut new_state = self.0;
        new_state &= !(0xFFFFFu128 << 75); // Clear worker positions
        new_state |= (self.0 & (0x3FFu128 << 75)) << 10; // Move player A workers to player B workers
        new_state |= (self.0 & (0x3FFu128 << 85)) >> 10; // Move player B workers to player A workers

        new_state ^= 1u128 << 95; // Flip player turn

        if new_state & (3u128 << 126) != 0 {
            new_state ^= 3u128 << 126; // Flip win bits
        }

        return GameState5x5Binary128bit(new_state);
    }
}

impl GameState5x5Binary128bit {
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

impl MinimaxReady for GameState5x5Binary128bit {
    fn sort_children_states(children_states: &mut Vec<Self>, depth: usize, cache: &mut MinimaxCache<Self>) {
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