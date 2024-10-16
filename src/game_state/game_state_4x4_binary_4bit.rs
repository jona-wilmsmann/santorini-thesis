use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, SantoriniEval, SantoriniState4x4, SimplifiedState};

use crate::game_state::utils::precompute_position_to_tile_id::precompute_position_to_tile_id;
use crate::game_state::utils::child_evaluation::gs4x4_child_evaluation;
use crate::game_state::utils::symmetric_simplified::gs4x4_symmetric_simplified;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;

/*
For this encoding, player A is always the active player
When converting to and from the generic game state, the block count needs to be looked at (if it's even, it is generic player A's turn)

Bits 0-63: 4 bits per tile, 16 tiles

For each tile:
- Bits 0-1: Height (0-3)
- Bit 2: Opponent present
- Bit 3: Player present
- Special case: Height 4 is represented as 1100 (This is acceptable because no worker can be placed on height 4)
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
                        if neighbor_column < 0 || neighbor_column >= 4 || (neighbor_row == row && neighbor_column == column) {
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

    pub fn get_generic_player_positions(&self) -> (usize, usize) {
        let player_a_bits = self.0 & Self::PLAYER_A_MASK;
        let player_b_bits = self.0 & Self::PLAYER_B_MASK;
        let cleaned_player_a_bit = player_a_bits & !(player_b_bits << 1);
        let cleaned_player_b_bit = player_b_bits & !(player_a_bits >> 1);

        let player_a_position = if cleaned_player_a_bit == 0 {
            16
        } else {
            cleaned_player_a_bit.trailing_zeros() as usize / 4
        };
        let player_b_position = if cleaned_player_b_bit == 0 {
            16
        } else {
            cleaned_player_b_bit.trailing_zeros() as usize / 4
        };

        return if self.is_player_a_turn() {
            (player_a_position, player_b_position)
        } else {
            (player_b_position, player_a_position)
        };
    }

    fn has_internal_player_a_won(&self) -> bool {
        let player_a_bits = self.0 & Self::PLAYER_A_MASK;
        if player_a_bits == 0 {
            return false;
        }
        let player_b_bits = self.0 & Self::PLAYER_B_MASK;
        let cleaned_player_a_bit = player_a_bits & !(player_b_bits << 1);
        let player_a_position = cleaned_player_a_bit.trailing_zeros() / 4;
        return ((self.0 >> (player_a_position * 4)) & 3) == 3;
    }

    fn has_internal_player_b_won(&self) -> bool {
        let player_b_bits = self.0 & Self::PLAYER_B_MASK;
        if player_b_bits == 0 {
            return false;
        }
        let player_a_bits = self.0 & Self::PLAYER_A_MASK;
        let cleaned_player_b_bit = player_b_bits & !(player_a_bits << 1);
        let player_b_position = cleaned_player_b_bit.trailing_zeros() / 4;
        return ((self.0 >> (player_b_position * 4)) & 3) == 3;
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

    #[inline(always)]
    fn is_player_a_turn(&self) -> bool {
        return if self.0 & Self::PLAYER_A_MASK != 0 {
            // Workers are placed
            let lowest_bit_mask = 0x1111111111111111;
            let block_count_even = (self.0 & lowest_bit_mask).count_ones() % 2 == 0;
            block_count_even
        } else {
            // If neither worker is placed, it is generic player A's turn.
            // If worker B is placed but worker A isn't, it must mean that generic player A has just placed it, and it is generic player B's turn.
            self.0 & Self::PLAYER_B_MASK == 0
        };
    }

    fn has_player_a_won(&self) -> bool {
        return if self.is_player_a_turn() {
            self.has_internal_player_a_won()
        } else {
            self.has_internal_player_b_won()
        };
    }

    fn has_player_b_won(&self) -> bool {
        return if self.is_player_a_turn() {
            self.has_internal_player_b_won()
        } else {
            self.has_internal_player_a_won()
        };
    }


    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<4, 4, 1>) -> Self {
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

        let mut binary_game_state = 0;

        for position in (0..16).rev() {
            binary_game_state <<= 4;
            let tile_id = Self::POSITION_TO_TILE_ID[position];
            let height = generic_game_state.get_tile_height(tile_id);

            let mut info;
            if height == 4 {
                info = 0b1100;
            } else {
                info = height as u64;
                if internal_player_a_tile == tile_id {
                    info |= 0b1000;
                } else if internal_player_b_tile == tile_id {
                    info |= 0b0100;
                }
            }
            binary_game_state |= info;
        }

        return Self(binary_game_state);
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<4, 4, 1> {
        let mut tile_heights = [[0; 4]; 4];
        let mut internal_player_a_tiles = None;
        let mut internal_player_b_tiles = None;

        let mut state = self.0;

        for position in 0..16 {
            let tile_id = Self::POSITION_TO_TILE_ID[position];
            let info = state & 0xF;
            state >>= 4;

            if info == 0b1100 {
                tile_heights[tile_id / 4][tile_id % 4] = 4;
            } else {
                tile_heights[tile_id / 4][tile_id % 4] = (info & 0x3) as u8;
                if info & 0x8 != 0 {
                    internal_player_a_tiles = Some([tile_id as u8]);
                } else if info & 0x4 != 0 {
                    internal_player_b_tiles = Some([tile_id as u8]);
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

        let player_a_bits = self.0 & Self::PLAYER_A_MASK;
        let player_b_bits = self.0 & Self::PLAYER_B_MASK;
        let cleaned_player_a_bit = player_a_bits & !(player_b_bits << 1);
        let cleaned_player_b_bit = player_b_bits & !(player_a_bits >> 1);

        let flipped_base_state = (self.0 & !cleaned_player_a_bit) + cleaned_player_b_bit;

        if player_a_bits == 0 {
            // No worker placed
            let mut new_worker_mask = 0b0100;
            for _ in 0..16 {
                // This check works because we are comparing the mask for the new flipped state (where the worker will be worker B)
                if new_worker_mask != cleaned_player_b_bit {
                    let new_state = flipped_base_state | new_worker_mask;
                    possible_next_states.push(Self(new_state));
                }
                new_worker_mask <<= 4;
            }
            return;
        }

        let player_a_position = (cleaned_player_a_bit.trailing_zeros() / 4) as usize;
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
        let movement_sub_result = state_with_padded_highest_bit - movement_height_threshold_mask;

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
            let build_sub_result = state_with_padded_highest_bit - build_height_threshold_mask;
            let mut valid_build_neighbors_mask = !build_sub_result & build_neighbor_mask & CARRY_MASK;

            let mut seen_build_positions = 0;
            loop {
                let new_build_positions = valid_build_neighbors_mask.trailing_zeros() / 4;
                if new_build_positions == 16 {
                    break;
                }

                let build_position = (new_build_positions + seen_build_positions) as usize;

                let mut new_state = (flipped_base_state | (0b0100 << (movement_position * 4))) + (1 << (build_position * 4));
                if (new_state >> (build_position * 4)) & 0b0100 != 0 {
                    // Special case for incrementing height from 3 to 4
                    new_state |= 0b1100 << (build_position * 4);
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

impl SantoriniEval for GameState4x4Binary4Bit {
    type SantoriniState = SantoriniState4x4;

    fn get_santorini_state(&self) -> Self::SantoriniState {
        let mut worker_a_position = 16;
        let mut worker_b_position = 16;
        let mut position_heights = [0; 16];

        let mut block_count = 0;

        let mut state = self.0;
        for position in 0..16 {
            let info = (state & 0xF) as u8;
            if info == 0b1100 {
                position_heights[position] = 4;
                block_count += 4;
            } else {
                let height = info & 0x3;
                position_heights[position] = height;
                block_count += height;
                if info & 0x8 != 0 {
                    worker_a_position = position as u8;
                } else if info & 0x4 != 0 {
                    worker_b_position = position as u8;
                }
            }
            state >>= 4;
        }

        let player_a_turn = if worker_a_position == 16 {
            true
        } else if worker_b_position == 16 {
            false
        } else {
            block_count % 2 == 0
        };

        return SantoriniState4x4 {
            position_heights,
            worker_a_position,
            worker_b_position,
            player_a_turn,
        };
    }

    fn get_child_evaluation(&self) -> f32 {
        let state = self.get_santorini_state();

        if state.worker_a_position != 16 && state.position_heights[state.worker_a_position as usize] == 3 {
            return f32::INFINITY;
        } else if state.worker_b_position != 16 && state.position_heights[state.worker_b_position as usize] == 3 {
            return f32::NEG_INFINITY;
        }

        return gs4x4_child_evaluation::get_child_evaluation(state);
    }
}

impl SimplifiedState for GameState4x4Binary4Bit {
    fn get_simplified_state(&self) -> Self {
        let (player_a_position, player_b_position) = self.get_generic_player_positions();

        let transposition_type = gs4x4_symmetric_simplified::PLAYER_A_POS_PLAYER_B_POS_TO_MIRROR_TYPE[player_a_position][player_b_position];

        let ccw_rotations = transposition_type as u64 % 4;
        let diagonal_mirroring = transposition_type >= 4;

        let mut new_state = match ccw_rotations {
            0 => self.0,
            1 => (self.0 << 16) | (self.0 >> 48),
            2 => (self.0 << 32) | (self.0 >> 32),
            3 => (self.0 << 48) | (self.0 >> 16),
            _ => panic!("Invalid rotation")
        };

        if diagonal_mirroring {
            let mut mirrored_height_information = 0;
            for original_pos in 0..16 {
                let mirrored_pos = gs4x4_symmetric_simplified::POS_TO_DIAGONALLY_MIRRORED_POS[original_pos];

                let original_tile = (new_state >> (original_pos * 4)) & 0xF;
                mirrored_height_information |= original_tile << (mirrored_pos * 4);
            }
            new_state = mirrored_height_information;
        }

        return Self(new_state);
    }

    fn is_simplified(&self) -> bool {
        let (player_a_position, player_b_position) = self.get_generic_player_positions();

        if player_a_position == 16 {
            return true;
        }

        for combination in gs4x4_symmetric_simplified::POSSIBLE_SIMPLIFIED_STATE_VARIANTS.iter() {
            if player_a_position == combination.player_a_position {
                if player_b_position == 16 {
                    return true;
                }
                for i in 0..combination.player_b_options {
                    if player_b_position == combination.player_b_positions[i] {
                        return true;
                    }
                }
                return false;
            }
        }
        return false;
    }
}