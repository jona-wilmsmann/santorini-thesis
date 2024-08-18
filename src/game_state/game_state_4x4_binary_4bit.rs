use std::fmt;
use std::fmt::Formatter;
use crate::game_state::GameState;

use crate::game_state::utils::precompute_position_to_tile_id::precompute_position_to_tile_id;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;

/*
Bits 0-63: 4 bits per tile, 16 tiles

For each tile:
- Bits 0-1: Height (0-3)
- Bit 2: Opponent present
- Bit 3: Player present
- Special case: Height 4 is represented as bits 0-2 being 111, in which case no opponent is present
- This is acceptable because the opponent being on a tile with height 3 means that the opponent already won
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

    fn has_player_a_won(&self) -> bool {
        // TODO: The 4bit game state does not have a proper encoding for win conditions
        panic!("Not implemented");
    }

    fn has_player_b_won(&self) -> bool {
        // TODO: The 4bit game state does not have a proper encoding for win conditions
        panic!("Not implemented");
    }


    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<4, 4, 1>) -> Self {
        let mut binary_game_state = 0;

        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            let height = if generic_game_state.tile_heights[i / 4][i % 4] == 4 { 7 } else { generic_game_state.tile_heights[i / 4][i % 4] };
            let player_present = generic_game_state.player_a_pieces[0] as usize == i;
            let opponent_present = generic_game_state.player_b_pieces[0] as usize == i;
            let info = (height as u64) | (player_present as u64) << 3 | (opponent_present as u64) << 2;
            binary_game_state |= info << (position * 4);
        }

        return Self(binary_game_state);
    }

    fn to_generic_game_state(self) -> GenericSantoriniGameState<4, 4, 1> {
        let mut tile_heights = [[0; 4]; 4];
        let mut player_a_tile = 0;
        let mut player_b_tile = 0;
        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            let info = (self.0 >> (position * 4)) & 0xF;
            tile_heights[i / 4][i % 4] = if info & 0x7 == 7 { 4 } else { (info & 0x3) as u8 };
            if info & 0x8 != 0 {
                player_a_tile = i as u8;
            } else if info & 0x4 != 0 && info & 0x7 != 7 {
                player_b_tile = i as u8;
            }
        }
        //TODO: Encoding for player_a_turn
        return GenericSantoriniGameState::<4, 4, 1>::new([player_a_tile], [player_b_tile], tile_heights, true).expect("Invalid game state");
    }

    fn get_children_states(self) -> Vec<Self> {
        return self.get_children_states_reuse_vec(Vec::with_capacity(32));
    }

    fn get_children_states_reuse_vec(&self, vec: Vec<Self>) -> Vec<Self> {
        let mut possible_next_states = vec;
        possible_next_states.clear();

        let player_a_bit = self.0 & Self::PLAYER_A_MASK;
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

                let mut new_state = ((self.0 ^ player_a_bit) | (0x8 << (movement_position * 4))) + (1 << (build_position * 4));
                // Special case for incrementing height from 3 to 4
                if new_state & (0x4 << (build_position * 4)) != 0 {
                    new_state |= 0x3 << (build_position * 4);
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

        return possible_next_states;
    }

    fn get_flipped_state(&self) -> Self {
        let player_a_bit = self.0 & Self::PLAYER_A_MASK;

        const PLAYER_B_HEIGHT_MASK: u64 = 0x3333333333333333;
        const PLAYER_B_ADDITION_MASK: u64 = 0x1111111111111111;
        let cleaned_player_b_state = self.0 & !((self.0 & PLAYER_B_HEIGHT_MASK) + PLAYER_B_ADDITION_MASK);
        let player_b_bit = cleaned_player_b_state & Self::PLAYER_B_MASK;

        if player_a_bit.count_ones() != 1 {
            panic!("Player A bit count is not 1");
        }
        if player_b_bit.count_ones() != 1 {
            panic!("Player B bit count is not 1");
        }

        let flipped_state = self.0 - (player_a_bit >> 1) + player_b_bit;
        return Self(flipped_state);
    }
}