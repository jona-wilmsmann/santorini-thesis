use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, MinimaxReady};
use crate::game_state::utils::static_evaluation::gs5x5_static_evaluation;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use crate::minimax::minimax_cache::MinimaxCache;

pub const fn precompute_bit_mask(base_mask: u128, base_mask_bit_size: usize, repeats: usize) -> u128 {
    let mut mask = 0u128;
    let mut i = 0;
    while i < repeats {
        mask <<= base_mask_bit_size;
        mask |= base_mask;
        i += 1;
    }
    return mask;
}

/*
Internally, it is always Player A's turn to move.

Bits 0-124: tile data (25 x 5 bits)
Bit 125: generic player B won
Bit 126: generic player A won
Bit 127: generic player A to move


For each tile:
- Bit 0-2: height
- Bit 3: worker B present
- Bit 4: worker A present
 */
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct GameState5x5Binary5bit(u128);

impl fmt::Display for GameState5x5Binary5bit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState5x5Binary5bit {
    const WORKER_NOT_PLACED: u8 = 25;
    const PLAYER_A_MASK: u128 = precompute_bit_mask(0b10000, 5, 25);
    const PLAYER_B_MASK: u128 = precompute_bit_mask(0b01000, 5, 25);


    const TILE_TO_NEIGHBOR_MASK: [u128; 25] = Self::precompute_tile_to_neighbor_mask();
    const fn precompute_tile_to_neighbor_mask() -> [u128; 25] {
        let mut neighbor_masks = [0; 25];
        let mut row: isize = 0;
        while row < 5 {
            let mut column = 0;
            while column < 5 {
                let tile_id = (row * 5 + column) as usize;
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
                        let neighbor_tile_id = neighbor_row * 5 + neighbor_column;
                        neighbor_masks[tile_id] |= 0b11111 << neighbor_tile_id * 5;
                        neighbor_column += 1;
                    }
                    neighbor_row += 1;
                }
                column += 1;
            }
            row += 1;
        }
        return neighbor_masks;
    }


    pub fn get_heights_and_generic_workers(&self) -> ([u8; 25], [u8; 2], [u8; 2]) {
        let mut tile_heights = [0; 25];
        let mut player_a_worker_tiles = [Self::WORKER_NOT_PLACED; 2];
        let mut player_b_worker_tiles = [Self::WORKER_NOT_PLACED; 2];

        let mut found_first_worker_a = false;
        let mut found_first_worker_b = false;

        let mut data = self.0;
        for tile_id in 0..25 {
            tile_heights[tile_id] = (data & 0b00111) as u8;

            if (data & 0b01000) != 0 {
                if !found_first_worker_b {
                    player_b_worker_tiles[0] = tile_id as u8;
                    found_first_worker_b = true;
                } else {
                    player_b_worker_tiles[1] = tile_id as u8;
                }
            }
            if (data & 0b10000) != 0 {
                if !found_first_worker_a {
                    player_a_worker_tiles[0] = tile_id as u8;
                    found_first_worker_a = true;
                } else {
                    player_a_worker_tiles[1] = tile_id as u8;
                }
            }
            data >>= 5;
        }

        return if self.is_player_a_turn() {
            (tile_heights, player_a_worker_tiles, player_b_worker_tiles)
        } else {
            (tile_heights, player_b_worker_tiles, player_a_worker_tiles)
        };
    }
}


impl GameState for GameState5x5Binary5bit {
    type RawValue = u128;
    type GenericGameState = GenericSantoriniGameState<5, 5, 2>;

    fn new(value: u128) -> Self {
        GameState5x5Binary5bit(value)
    }

    fn raw_value(&self) -> u128 {
        return self.0;
    }

    fn is_player_a_turn(&self) -> bool {
        return self.0 & (1 << 127) != 0;
    }

    fn has_player_a_won(&self) -> bool {
        return self.0 & (1 << 126) != 0;
    }

    fn has_player_b_won(&self) -> bool {
        return self.0 & (1 << 125) != 0;
    }

    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<5, 5, 2>) -> Self {
        let generic_player_a_worker_tiles = generic_game_state.player_a_workers.unwrap_or([Self::WORKER_NOT_PLACED; 2]);
        let generic_player_b_worker_tiles = generic_game_state.player_b_workers.unwrap_or([Self::WORKER_NOT_PLACED; 2]);

        let internal_player_a_worker_tiles = if generic_game_state.player_a_turn {
            generic_player_a_worker_tiles
        } else {
            generic_player_b_worker_tiles
        };
        let internal_player_b_worker_tiles = if generic_game_state.player_a_turn {
            generic_player_b_worker_tiles
        } else {
            generic_player_a_worker_tiles
        };

        let mut data = 0u128;
        for tile_id in (0..25).rev() {
            data <<= 5;
            data |= generic_game_state.get_tile_height(tile_id) as u128;
            if internal_player_b_worker_tiles[0] == tile_id as u8 || internal_player_b_worker_tiles[1] == tile_id as u8 {
                data |= 1 << 3;
            }
            if internal_player_a_worker_tiles[0] == tile_id as u8 || internal_player_a_worker_tiles[1] == tile_id as u8 {
                data |= 1 << 4;
            }
        }

        if generic_game_state.has_player_b_won() {
            data |= 1 << 125;
        }
        if generic_game_state.has_player_a_won() {
            data |= 1 << 126;
        }
        if generic_game_state.player_a_turn {
            data |= 1 << 127;
        }
        return GameState5x5Binary5bit(data);
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<5, 5, 2> {
        let (tile_heights, player_a_workers, player_b_workers) = self.get_heights_and_generic_workers();

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

        return GenericSantoriniGameState::<5, 5, 2>::new(generic_player_a_workers, generic_player_b_workers, generic_tile_heights, self.is_player_a_turn())
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

        let mut player_a_bits = self.0 & Self::PLAYER_A_MASK;
        let player_b_bits = self.0 & Self::PLAYER_B_MASK;

        let flipped_base_state = ((self.0 & !Self::PLAYER_A_MASK) + player_b_bits) ^ (1 << 127);

        if player_a_bits == 0 {
            // Workers not placed yet
            let mut state_w1 = self.0;
            for worker_1_tile_id in 0..25 {
                // If this is not 0, the other player has already placed workers here
                if state_w1 & 0b11111 == 0 {
                    let mut state_w2 = state_w1 >> 5;
                    for worker_2_tile_id in (worker_1_tile_id + 1)..25 {
                        if state_w2 & 0b11111 == 0 {
                            let mut new_state = flipped_base_state;
                            new_state |= 0b01000 << (worker_1_tile_id * 5);
                            new_state |= 0b01000 << (worker_2_tile_id * 5);
                            possible_next_states.push(GameState5x5Binary5bit(new_state));
                        }
                        state_w2 >>= 5;
                    }
                }
                state_w1 >>= 5;
            }
            return;
        }

        // For each tile, sets bit 4 to 1 and sets bit 3 to 1 if any worker is present
        let padded_state = self.0 | Self::PLAYER_A_MASK | (player_a_bits >> 1);

        // Workers are placed
        let player_a_worker_1_tile = player_a_bits.trailing_zeros() as usize / 5;
        player_a_bits >>= (player_a_worker_1_tile + 1) * 5;
        let player_a_worker_2_tile = player_a_bits.trailing_zeros() as usize / 5 + player_a_worker_1_tile + 1;


        for (moving_worker_tile, passive_worker_tile) in [(player_a_worker_1_tile, player_a_worker_2_tile), (player_a_worker_2_tile, player_a_worker_1_tile)] {
            let flipped_base_state_with_passive_worker = flipped_base_state | (0b01000 << (passive_worker_tile * 5));

            let moving_worker_height = self.0 >> (moving_worker_tile * 5) & 0b00111;

            const CARRY_MASK: u128 = precompute_bit_mask(0b10000, 5, 25);
            const HEIGHT_2_MASK: u128 = precompute_bit_mask(2, 5, 25);
            const HEIGHT_3_MASK: u128 = precompute_bit_mask(3, 5, 25);
            const HEIGHT_4_MASK: u128 = precompute_bit_mask(4, 5, 25);
            let movement_height_threshold_mask = match moving_worker_height {
                0 => HEIGHT_2_MASK,
                1 => HEIGHT_3_MASK,
                _ => HEIGHT_4_MASK,
            };

            // The highest bit of each tile is 1 before the subtraction, and will become 0 if the remaining tile bits are below the threshold
            let movement_sub_result = padded_state - movement_height_threshold_mask;

            let movement_neighbor_mask = Self::TILE_TO_NEIGHBOR_MASK[moving_worker_tile];
            let mut valid_movement_neighbors_mask = (!movement_sub_result) & movement_neighbor_mask & CARRY_MASK;

            let mut seen_movement_positions = 0;
            loop {
                let new_movement_positions = valid_movement_neighbors_mask.trailing_zeros() as usize / 5;
                if new_movement_positions == 25 {
                    // No valid targets
                    break;
                }
                let movement_position = new_movement_positions + seen_movement_positions;


                // Building where the worker moved from is always possible
                let mut new_state = (flipped_base_state_with_passive_worker | (0b01000 << (movement_position * 5))) + (1 << (moving_worker_tile * 5));
                if (new_state >> movement_position * 5) & 0b00111 == 3 {
                    if self.is_player_a_turn() {
                        new_state |= 1 << 126;
                    } else {
                        new_state |= 1 << 125;
                    }
                }
                possible_next_states.push(GameState5x5Binary5bit(new_state));


                let build_neighbor_mask = Self::TILE_TO_NEIGHBOR_MASK[movement_position];
                let build_sub_result = padded_state - HEIGHT_4_MASK;
                let mut valid_build_neighbors_mask = (!build_sub_result) & build_neighbor_mask & CARRY_MASK;

                let mut seen_build_positions = 0;
                loop {
                    let new_build_positions = valid_build_neighbors_mask.trailing_zeros() as usize / 5;
                    if new_build_positions == 25 {
                        // No valid targets
                        break;
                    }
                    let build_position = new_build_positions + seen_build_positions;


                    let mut new_state = (flipped_base_state_with_passive_worker | (0b01000 << (movement_position * 5))) + (1 << (build_position * 5));
                    if (new_state >> movement_position * 5) & 0b00111 == 3 {
                        if self.is_player_a_turn() {
                            new_state |= 1 << 126;
                        } else {
                            new_state |= 1 << 125;
                        }
                    }
                    possible_next_states.push(GameState5x5Binary5bit(new_state));


                    if new_build_positions >= 24 {
                        break;
                    }
                    valid_build_neighbors_mask >>= (new_build_positions + 1) * 5;
                    seen_build_positions += new_build_positions + 1;
                }


                if new_movement_positions >= 24 {
                    break;
                }
                valid_movement_neighbors_mask >>= (new_movement_positions + 1) * 5;
                seen_movement_positions += new_movement_positions + 1;
            }
        }
    }
}


impl MinimaxReady for GameState5x5Binary5bit {
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
        let (tile_heights, player_a_workers, player_b_workers) = self.get_heights_and_generic_workers();
        return gs5x5_static_evaluation::get_static_evaluation(tile_heights, player_a_workers, player_b_workers, self.is_player_a_turn());
    }
}