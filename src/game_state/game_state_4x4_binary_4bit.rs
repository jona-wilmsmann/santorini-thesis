use std::fmt;
use std::fmt::Formatter;
use crate::game_state::GameState;

use crate::game_state::utils::precompute_position_to_tile_id::precompute_position_to_tile_id;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;

/*
For this encoding, player A is always the active player
When converting to and from the generic game state, the block count needs to be looked at (if it's even, it is generic player A's turn)

Bits 0-63: 4 bits per tile, 16 tiles

For each tile:
- Bits 0-1: Height (0-3)
- Bit 2: Opponent present
- Bit 3: Player present
- Special case: Height 4 is represented as bits 0-2 being 111, in which case no opponent is present
- Special case: Opponent on height 3 tile is represented as bits 0-3 being 1111
 */
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct GameState4x4Binary4Bit(u64);

impl fmt::Display for GameState4x4Binary4Bit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState4x4Binary4Bit {
    /*
    12 10 9  8
    13 15 11 6
    14 3  7  5
    0  1  2  4
     */
    const TILE_ID_TO_POSITION: [usize; 16] = [0, 1, 2, 4, 14, 3, 7, 5, 13, 15, 11, 6, 12, 10, 9, 8];
    const POSITION_TO_TILE_ID: [usize; 16] = precompute_position_to_tile_id(Self::TILE_ID_TO_POSITION);
    const POSITION_TO_NEIGHBOR_MASK: [u64; 16] = Self::precompute_position_to_neighbor_mask();
    const fn precompute_position_to_neighbor_mask() -> [u64; 16] {
        let mut neighbor_masks = [0; 16];
        let mut row: isize = 0;
        while row < 4 {
            let mut column = 0;
            while column < 4 {
                let tile_id = row * 4 + column;
                let position = Self::TILE_ID_TO_POSITION[tile_id as usize];
                let mut neighbor_row = row - 1;
                while neighbor_row <= row + 1 {
                    if neighbor_row < 0 || neighbor_row >= 4 {
                        neighbor_row += 1;
                        continue;
                    }
                    let mut neighbor_column = column - 1;
                    while neighbor_column <= column + 1 {
                        if neighbor_column < 0 || neighbor_column >= 4 || (neighbor_row == row && neighbor_column == column){
                            neighbor_column += 1;
                            continue;
                        }
                        let neighbor_tile_id = neighbor_row * 4 + neighbor_column;
                        let neighbor_position = Self::TILE_ID_TO_POSITION[neighbor_tile_id as usize];
                        neighbor_masks[position] |= 0xF << neighbor_position * 4;
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

    const PLAYER_A_MASK: u64 = 0x8888888888888888;
    const PLAYER_B_MASK: u64 = 0x4444444444444444;

    fn get_block_count(&self) -> u64 {
        let mut block_count = 0;
        let mut state = self.0;
        for _ in 0..16 {
            let tile_info = state & 0xF;
            if tile_info == 0b1111 {
                // Special case for player B on height 3
                block_count += 3;
            } else if tile_info == 0b0111 {
                // Special case for height 4
                block_count += 4;
            } else {
                block_count += tile_info & 0x3;
            }
            state >>= 4;
        }
        return block_count;
    }

    fn has_internal_player_a_won(&self) -> bool {
        let player_a_bit = self.0 & Self::PLAYER_A_MASK;
        if player_a_bit.count_ones() != 1 {
            // Either no workers are placed or player B has won
            return false;
        }
        let player_a_position = (player_a_bit.trailing_zeros() / 4) as usize;

        let player_a_height = (self.0 >> (player_a_position * 4)) & 0x3;

        return player_a_height == 3;
    }

    fn has_internal_player_b_won(&self) -> bool {
        let player_a_bit = self.0 & Self::PLAYER_A_MASK;

        // There can only be two player A bits if the special case for player B on height 3 is present
        return player_a_bit.count_ones() == 2;
    }
}

impl GameState for GameState4x4Binary4Bit {
    type RawValue = u64;
    type GenericGameState = GenericSantoriniGameState<4, 4, 1>;

    fn new(value: u64) -> Self {
        Self(value)
    }

    fn raw_value(&self) -> u64 {
        self.0
    }

    fn is_player_a_turn(&self) -> bool {
        let block_count = self.get_block_count();
        return if self.0 & Self::PLAYER_A_MASK != 0 {
            // Workers are placed
            block_count % 2 == 0
        } else {
            // If neither worker is placed, it is generic player A's turn.
            // If worker B is placed but worker A isn't, it must mean that generic player A has just placed it, and it is generic player B's turn.
            self.0 & Self::PLAYER_B_MASK == 0
        }
    }

    fn has_player_a_won(&self) -> bool {
        return if self.is_player_a_turn() {
            self.has_internal_player_a_won()
        } else {
            self.has_internal_player_b_won()
        }
    }

    fn has_player_b_won(&self) -> bool {
        return if self.is_player_a_turn() {
            self.has_internal_player_b_won()
        } else {
            self.has_internal_player_a_won()
        }
    }


    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<4, 4, 1>) -> Self {
        let mut binary_game_state = 0;

        let generic_player_a_tile = generic_game_state.player_a_workers.unwrap_or([u8::MAX])[0] as usize;
        let generic_player_b_tile = generic_game_state.player_b_workers.unwrap_or([u8::MAX])[0] as usize;
        let internal_player_a_tile = if generic_game_state.player_a_turn {
            generic_player_a_tile
        } else {
            generic_player_b_tile
        };
        let internal_player_b_tile = if generic_game_state.player_a_turn {
            generic_player_b_tile
        } else {
            generic_player_a_tile
        };


        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            let height = if generic_game_state.tile_heights[i / 4][i % 4] == 4 { 7 } else { generic_game_state.tile_heights[i / 4][i % 4] };
            let player_present = internal_player_a_tile == i;
            let opponent_present = internal_player_b_tile == i;
            let info = if height == 3 && opponent_present {
                0b1111
            } else {
                (height as u64) | (player_present as u64) << 3 | (opponent_present as u64) << 2
            };
            binary_game_state |= info << (position * 4);
        }

        return Self(binary_game_state);
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<4, 4, 1> {
        let mut tile_heights = [[0; 4]; 4];
        let mut internal_player_a_tiles = None;
        let mut internal_player_b_tiles = None;

        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            let info = (self.0 >> (position * 4)) & 0xF;

            if info == 0b1111 {
                tile_heights[i / 4][i % 4] = 3;
                internal_player_b_tiles = Some([i as u8]);

            } else if info == 0b0111 {
                tile_heights[i / 4][i % 4] = 4;

            } else {
                tile_heights[i / 4][i % 4] = (info & 0x3) as u8;
                if info & 0x8 != 0 {
                    internal_player_a_tiles = Some([i as u8]);
                } else if info & 0x4 != 0 {
                    internal_player_b_tiles = Some([i as u8]);
                }
            }
        }

        let generic_player_a_turn = self.is_player_a_turn();
        let generic_player_a_workers = if generic_player_a_turn {
            internal_player_a_tiles
        } else {
            internal_player_b_tiles
        };
        let generic_player_b_workers = if generic_player_a_turn {
            internal_player_b_tiles
        } else {
            internal_player_a_tiles
        };

        return GenericSantoriniGameState::<4, 4, 1>::new(generic_player_a_workers, generic_player_b_workers, tile_heights, generic_player_a_turn).expect("Invalid game state");
    }

    fn get_children_states(&self) -> Vec<Self> {
        let mut possible_next_states = Vec::with_capacity(32);
        self.get_children_states_reuse_vec(&mut possible_next_states);
        return possible_next_states;
    }

    fn get_children_states_reuse_vec(&self, possible_next_states: &mut Vec<Self>) {
        debug_assert!(!self.has_player_a_won());
        debug_assert!(!self.has_player_b_won());

        possible_next_states.clear();

        const PLAYER_B_HEIGHT_MASK: u64 = 0x3333333333333333;
        const PLAYER_B_ADDITION_MASK: u64 = 0x1111111111111111;
        let cleaned_player_b_state = self.0 & !((self.0 & PLAYER_B_HEIGHT_MASK) + PLAYER_B_ADDITION_MASK);
        let player_b_bit = cleaned_player_b_state & Self::PLAYER_B_MASK;
        // Current player B becomes player A, player B is cleared so that it can be set later
        let flipped_base_state = (self.0 & !Self::PLAYER_A_MASK) | (player_b_bit << 1);



        let player_a_bit = self.0 & Self::PLAYER_A_MASK;

        if player_a_bit == 0 {
            let player_b_position = if player_b_bit == 0 { usize::MAX } else { (player_b_bit.trailing_zeros() / 4) as usize };
            // No workers placed
            for i in 0..16 {
                if i == player_b_position {
                    // Worker B is placed
                    continue;
                }
                let new_state = flipped_base_state | (0b0100 << (i * 4));
                possible_next_states.push(Self(new_state));
            }
            return;
        }

        let player_a_position = (player_a_bit.trailing_zeros() / 4) as usize;
        let movement_neighbor_mask = Self::POSITION_TO_NEIGHBOR_MASK[player_a_position];

        let player_a_height = (self.0 >> (player_a_position * 4)) & 0x3;
        let movement_height_threshold_mask = match player_a_height {
            0 => 0x2222222222222222,
            1 => 0x3333333333333333,
            _ => 0x4444444444444444,
        };

        let state_with_padded_highest_bit = self.0 | 0x8888888888888888;
        const CARRY_MASK: u64 = 0x8888888888888888;

        // The highest bit of each tile is 1 before the subtraction, and will become 0 if the remaining tile bits are below the threshold
        let movement_sub_result = state_with_padded_highest_bit.wrapping_sub(movement_height_threshold_mask);

        let mut valid_movement_neighbors_mask = (!movement_sub_result) & movement_neighbor_mask & CARRY_MASK;

        let mut seen_movement_positions = 0;
        loop {
            let new_movement_positions = valid_movement_neighbors_mask.trailing_zeros() / 4;
            if new_movement_positions == 16 {
                break;
            }

            let movement_position = (new_movement_positions + seen_movement_positions) as usize;

            let build_neighbor_mask = Self::POSITION_TO_NEIGHBOR_MASK[movement_position];
            let build_height_threshold_mask = 0x4444444444444444;
            let build_sub_result = state_with_padded_highest_bit.wrapping_sub(build_height_threshold_mask);
            let mut valid_build_neighbors_mask = !build_sub_result & build_neighbor_mask & CARRY_MASK;

            let mut seen_build_positions = 0;
            loop {
                let new_build_positions = valid_build_neighbors_mask.trailing_zeros() / 4;
                if new_build_positions == 16 {
                    break;
                }

                let build_position = (new_build_positions + seen_build_positions) as usize;

                let mut new_state = (flipped_base_state | (0b0100 << (movement_position * 4))) + (1 << (build_position * 4));
                if (new_state >> (movement_position * 4)) & 0b0011 == 0b0011 {
                    // Active player reached height 3. They are player B in the flipped state, therefore the special case for player B on height 3 is used
                    new_state |= 0b1111 << (movement_position * 4);
                }
                if (new_state >> (build_position * 4)) & 0b0100 != 0 {
                    // Special case for incrementing height from 3 to 4
                    new_state |= 0b0011 << (build_position * 4);
                }


                possible_next_states.push(Self(new_state));

                if new_build_positions >= 15 {
                    break;
                }
                valid_build_neighbors_mask >>= (new_build_positions + 1) * 4;
                seen_build_positions += new_build_positions + 1;
            }

            if new_movement_positions >= 15 {
                break;
            }
            valid_movement_neighbors_mask >>= (new_movement_positions + 1) * 4;
            seen_movement_positions += new_movement_positions + 1;
        }
    }
}