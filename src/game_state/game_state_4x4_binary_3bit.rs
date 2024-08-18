use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use once_cell::sync::Lazy;
use crate::game_state::{ContinuousBlockId, ContinuousId, GameState, SimplifiedState, StaticEvaluation};
use crate::game_state::utils::precompute_position_to_tile_id::precompute_position_to_tile_id;
use crate::game_state::utils::get_binomial_coefficient::get_binomial_coefficient;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;

/*
Bits 0-47: 3 bits per tile, 16 tiles
Bits 48-51: Player A position
Bits 52-55: Player B position
Bits 56-60: Unused
Bit 61: Player A's turn (bool)
Bit 62: Player B has won (because they have reached height 3)
Bit 63: Player A has won (because they have reached height 3)

For each tile:
- Bits 0-2: Height (0-4)
 */
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct GameState4x4Binary3Bit(u64);

impl fmt::Display for GameState4x4Binary3Bit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState4x4Binary3Bit {
    /*
    12 10 9  8
    13 15 11 6
    14 3  7  5
    0  1  2  4
     */
    const TILE_ID_TO_POSITION: [usize; 16] = [0, 1, 2, 4, 14, 3, 7, 5, 13, 15, 11, 6, 12, 10, 9, 8];
    const POSITION_TO_TILE_ID: [usize; 16] = precompute_position_to_tile_id(Self::TILE_ID_TO_POSITION);
    const NO_NEIGHBOR: usize = usize::MAX;
    const POSITION_TO_NEIGHBORS: [[usize; 8]; 16] = Self::precompute_position_to_neighbors();
    const fn precompute_position_to_neighbors() -> [[usize; 8]; 16] {
        let mut position_to_neighbors = [[Self::NO_NEIGHBOR; 8]; 16];

        let mut row: isize = 0;
        while row < 4 {
            let mut column = 0;
            while column < 4 {
                let tile_id = row * 4 + column;
                let position = Self::TILE_ID_TO_POSITION[tile_id as usize];
                let mut position_neighbor_index = 0;

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
                        position_to_neighbors[position][position_neighbor_index] = neighbor_position;
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

    fn get_player_a_position(&self) -> u64 {
        return (self.0 >> 48) & 0xF;
    }

    fn get_player_b_position(&self) -> u64 {
        return (self.0 >> 52) & 0xF;
    }

    fn get_position_heights(&self) -> [u8; 16] {
        let mut position_heights = [0; 16];

        let mut data = self.0;
        for i in 0..16 {
            position_heights[i] = (data & 0x7) as u8;
            data >>= 3;
        }
        return position_heights;
    }

    fn is_player_a_turn(&self) -> bool {
        return self.0 & (1 << 61) != 0;
    }
}

impl GameState for GameState4x4Binary3Bit {
    type RawValue = u64;
    type GenericGameState = GenericSantoriniGameState<4, 4, 1>;

    fn new(binary_game_state: u64) -> Self {
        return Self(binary_game_state);
    }

    fn raw_value(&self) -> u64 {
        self.0
    }

    fn has_player_a_won(&self) -> bool {
        return self.0 & (1 << 63) != 0;
    }

    fn has_player_b_won(&self) -> bool {
        return self.0 & (1 << 62) != 0;
    }

    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<4, 4, 1>) -> Self {
        let mut binary_game_state = 0;
        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            let height = generic_game_state.get_tile_height(i);
            binary_game_state |= (height as u64) << (position * 3);
        }

        // TODO: Handle workers not being placed yet

        let player_a_tile = generic_game_state.player_a_workers[0] as usize;
        let player_b_tile = generic_game_state.player_b_workers[0] as usize;
        let player_a_position = Self::TILE_ID_TO_POSITION[player_a_tile] as u64;
        let player_b_position = Self::TILE_ID_TO_POSITION[player_b_tile] as u64;

        binary_game_state |= player_a_position << 48;
        binary_game_state |= player_b_position << 52;

        if generic_game_state.get_tile_height(player_a_tile) == 3 {
            binary_game_state |= 1 << 63;
        }
        if generic_game_state.get_tile_height(player_b_tile) == 3 {
            binary_game_state |= 1 << 62;
        }

        if generic_game_state.player_a_turn {
            binary_game_state |= 1 << 61;
        }

        return Self(binary_game_state);
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<4, 4, 1> {
        let position_heights = self.get_position_heights();
        let mut tile_heights = [[0; 4]; 4];
        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            tile_heights[i / 4][i % 4] = position_heights[position];
        }
        // Convert position to tile
        let player_a_tile = Self::POSITION_TO_TILE_ID[self.get_player_a_position() as usize] as u8;
        let player_b_tile = Self::POSITION_TO_TILE_ID[self.get_player_b_position() as usize] as u8;

        return GenericSantoriniGameState::<4, 4, 1>::new([player_a_tile], [player_b_tile], tile_heights, self.is_player_a_turn())
            .expect("Invalid game state");
    }

    fn get_children_states(&self) -> Vec<Self> {
        return self.get_children_states_reuse_vec(Vec::with_capacity(32));
    }

    fn get_children_states_reuse_vec(&self, vec: Vec<Self>) -> Vec<Self> {
        debug_assert!(!self.has_player_a_won());
        debug_assert!(!self.has_player_b_won());

        let mut possible_next_states = vec;
        possible_next_states.clear();

        let position_heights = self.get_position_heights();
        let is_player_a_turn = self.is_player_a_turn();

        let moving_player_position = if is_player_a_turn {
            self.get_player_a_position() as usize
        } else {
            self.get_player_b_position() as usize
        };
        let other_player_position = if is_player_a_turn {
            self.get_player_b_position() as usize
        } else {
            self.get_player_a_position() as usize
        };

        let moving_player_bit_offset = if is_player_a_turn {
            48
        } else {
            52
        };

        let moving_player_height = position_heights[moving_player_position];
        let max_movement_height = match moving_player_height {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => panic!("Can't get children for a game state that is already won")
        };

        // Clearing the moving player position and flipping the turn
        let new_state_base = (self.0 & !(0xF << moving_player_bit_offset)) ^ (1 << 61);


        for movement_position in Self::POSITION_TO_NEIGHBORS[moving_player_position] {
            if movement_position == Self::NO_NEIGHBOR {
                break;
            }
            if movement_position == other_player_position {
                continue;
            }
            let movement_height = position_heights[movement_position];
            if movement_height > max_movement_height {
                continue;
            }

            for build_position in Self::POSITION_TO_NEIGHBORS[movement_position] {
                if build_position == Self::NO_NEIGHBOR {
                    break;
                }
                if build_position == other_player_position {
                    continue;
                }
                let build_height = position_heights[build_position];
                if build_height >= 4 {
                    continue;
                }

                let mut new_state = new_state_base;
                new_state |= (movement_position as u64) << moving_player_bit_offset;
                new_state += 1 << (build_position * 3);
                if movement_height == 3 {
                    if is_player_a_turn {
                        new_state |= 1 << 63;
                    } else {
                        new_state |= 1 << 62;
                    }
                }

                possible_next_states.push(Self(new_state));
            }
        }

        return possible_next_states;
    }

    fn get_flipped_state(&self) -> Self {
        let mut flipped_state = self.0;
        let player_a_position = self.get_player_a_position();
        let player_b_position = self.get_player_b_position();

        // Clear the player positions
        flipped_state &= !(0xFF << 48);
        flipped_state |= player_a_position << 52;
        flipped_state |= player_b_position << 48;
        // Flip the winning bits
        if flipped_state & (3 << 62) != 0 {
            flipped_state ^= 3 << 62;
        }
        // Flip the turn
        flipped_state ^= 1 << 61;
        return Self(flipped_state);
    }
}


impl GameState4x4Binary3Bit {
    const DISTANCE_TO_STATIC_VALUATION: [f32; 4] = [5.0, 2.0, 1.0, 0.5];
    const HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION: [[f32; 5]; 3] = [
        [1.0, 1.5, -1.0, 0.0, -1.0], //Start height 0
        [1.0, 1.5, 2.0, 0.5, -1.0], //Start height 1
        [1.0, 1.5, 2.0, 3.0, -1.0], //Start height 2
    ];

    const POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION: [[[[f32; 5]; 3]; 16]; 16] =
        Self::precompute_position_to_position_to_height_to_height_to_valuation();
    const fn precompute_position_to_position_to_height_to_height_to_valuation() -> [[[[f32; 5]; 3]; 16]; 16] {
        let mut position_to_position_to_height_to_height_to_valuation = [[[[0.0; 5]; 3]; 16]; 16];

        let mut i = 0;
        while i < 16 {
            let row_i = i / 4;
            let column_i = i % 4;
            let position_i = Self::TILE_ID_TO_POSITION[i];
            let mut j = 0;
            while j < 16 {
                let row_j = j / 4;
                let column_j = j % 4;
                let position_j = Self::TILE_ID_TO_POSITION[j];

                let row_distance = if row_i > row_j { row_i - row_j } else { row_j - row_i };
                let column_distance = if column_i > column_j { column_i - column_j } else { column_j - column_i };
                let distance = if row_distance > column_distance { row_distance } else { column_distance };

                let mut start_height = 0;
                while start_height <= 2 {
                    let mut neighbor_height = 0;
                    while neighbor_height <= 4 {
                        let height_valuation = Self::HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION[start_height][neighbor_height];
                        let distance_valuation = Self::DISTANCE_TO_STATIC_VALUATION[distance];
                        position_to_position_to_height_to_height_to_valuation[position_i][position_j][start_height][neighbor_height] = height_valuation * distance_valuation;
                        neighbor_height += 1;
                    }

                    start_height += 1;
                }
                j += 1;
            }
            i += 1;
        }

        return position_to_position_to_height_to_height_to_valuation;
    }
}

impl StaticEvaluation for GameState4x4Binary3Bit {
    fn get_static_evaluation(&self) -> f32 {
        if self.has_player_a_won() {
            return f32::MAX;
        } else if self.has_player_b_won() {
            return f32::MIN;
        }

        let player_a_position = self.get_player_a_position() as usize;
        let player_b_position = self.get_player_b_position() as usize;
        let position_heights = self.get_position_heights();
        let mut valuation = 0.0;

        for i in 0..16 {
            valuation += Self::POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[player_a_position][i][position_heights[player_a_position] as usize][position_heights[i] as usize];
            valuation -= Self::POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[player_b_position][i][position_heights[player_b_position] as usize][position_heights[i] as usize];
        }

        return valuation;
    }
}

/*
0 => 0 deg ccw
1 => 90 deg ccw
2 => 180 deg ccw
3 => 270 deg ccw
4 => 0 deg ccw + diagonal mirror
5 => 90 deg ccw + diagonal mirror
6 => 180 deg ccw + diagonal mirror
7 => 270 deg ccw + diagonal mirror
 */
type SymmetricMirrorType = u8;

impl GameState4x4Binary3Bit {
    const POS_TO_DIAGONALLY_MIRRORED_POS: [u64; 16] = Self::precompute_pos_to_diagonally_mirrored_pos();
    const fn precompute_pos_to_diagonally_mirrored_pos() -> [u64; 16] {
        let mut pos_to_diagonally_mirrored_pos = [0; 16];
        let mut tile_id = 0;
        while tile_id < 16 {
            let row = tile_id / 4;
            let column = tile_id % 4;
            let diagonally_mirrored_row = column;
            let diagonally_mirrored_column = row;
            let diagonally_mirrored_tile_id = diagonally_mirrored_row * 4 + diagonally_mirrored_column;

            let pos = Self::TILE_ID_TO_POSITION[tile_id];
            let diagonally_mirrored_pos = Self::TILE_ID_TO_POSITION[diagonally_mirrored_tile_id];
            pos_to_diagonally_mirrored_pos[pos] = diagonally_mirrored_pos as u64;

            tile_id += 1;
        }
        return pos_to_diagonally_mirrored_pos;
    }

    const fn get_rotated_tile_id(tile_id: usize, ccw_90_rotations: usize) -> usize {
        let mut new_tile_id = tile_id;
        const ROTATION_MAP: [usize; 16] = [3, 7, 11, 15, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12];
        let mut i = 0;
        while i < ccw_90_rotations {
            new_tile_id = ROTATION_MAP[new_tile_id];
            i += 1;
        }

        return new_tile_id;
    }

    const PLAYER_A_POS_PLAYER_B_POS_TO_MIRROR_TYPE: [[SymmetricMirrorType; 16]; 16] =
        Self::precompute_mirror_types();

    const fn precompute_mirror_types() -> [[SymmetricMirrorType; 16]; 16] {
        let mut player_a_pos_player_b_pos_to_mirror_type = [[0; 16]; 16];

        let mut player_a_tile = 0;
        while player_a_tile < 16 {
            let player_a_pos = Self::TILE_ID_TO_POSITION[player_a_tile];
            let mut player_b_tile = 0;
            while player_b_tile < 16 {
                let player_b_pos = Self::TILE_ID_TO_POSITION[player_b_tile];

                let ccw_rotations = match player_a_tile {
                    0 | 1 | 4 | 5 => 0,
                    2 | 3 | 6 | 7 => 3,
                    10 | 11 | 14 | 15 => 2,
                    8 | 9 | 12 | 13 => 1,
                    _ => panic!("Invalid tile id")
                };

                let player_a_tile_rotated = Self::get_rotated_tile_id(player_a_tile, ccw_rotations);
                let diagonal_mirroring = if player_a_tile_rotated == 4 {
                    true
                } else if player_a_tile_rotated == 1 {
                    false
                } else {
                    let player_b_tile_rotated = Self::get_rotated_tile_id(player_b_tile, ccw_rotations);
                    match player_b_tile_rotated {
                        4 | 8 | 9 | 12 | 13 | 14 => true,
                        _ => false
                    }
                };


                let mirror_type: SymmetricMirrorType = ccw_rotations as u8 + if diagonal_mirroring { 4 } else { 0 };
                player_a_pos_player_b_pos_to_mirror_type[player_a_pos][player_b_pos] = mirror_type;

                player_b_tile += 1;
            }
            player_a_tile += 1;
        }

        return player_a_pos_player_b_pos_to_mirror_type;
    }
}

impl SimplifiedState for GameState4x4Binary3Bit {
    fn get_simplified_state(&self) -> Self {
        let height_information = self.0 & 0xFFFFFFFFFFFF;
        let status_information = self.0 & (0xFF << 56);
        let player_a_position = self.get_player_a_position();
        let player_b_position = self.get_player_b_position();

        let transposition_type = Self::PLAYER_A_POS_PLAYER_B_POS_TO_MIRROR_TYPE[player_a_position as usize][player_b_position as usize];

        let ccw_rotations = transposition_type as u64 % 4;
        let diagonal_mirroring = transposition_type >= 4;

        let mut new_height_information = match ccw_rotations {
            0 => height_information,
            1 => ((height_information & 0xFFFFFFFFF) << 12) | (height_information >> 36),
            2 => ((height_information & 0xFFFFFF) << 24) | (height_information >> 24),
            3 => ((height_information & 0xFFF) << 36) | (height_information >> 12),
            _ => panic!("Invalid rotation")
        };

        if diagonal_mirroring {
            let mut mirrored_height_information = 0;
            for original_pos in 0..16 {
                let mirrored_pos = Self::POS_TO_DIAGONALLY_MIRRORED_POS[original_pos];

                let original_height = (new_height_information >> (original_pos * 3)) & 0x7;
                mirrored_height_information |= original_height << (mirrored_pos * 3);
            }
            new_height_information = mirrored_height_information;
        }

        let mut new_player_a_position = (player_a_position + ccw_rotations * 4) % 16;
        let mut new_player_b_position = (player_b_position + ccw_rotations * 4) % 16;
        if diagonal_mirroring {
            new_player_a_position = Self::POS_TO_DIAGONALLY_MIRRORED_POS[new_player_a_position as usize];
            new_player_b_position = Self::POS_TO_DIAGONALLY_MIRRORED_POS[new_player_b_position as usize];
        }

        let new_state = new_height_information | (new_player_a_position << 48) | (new_player_b_position << 52) | status_information;

        return Self(new_state);
    }

    fn is_simplified(&self) -> bool {
        let player_a_position = self.get_player_a_position() as usize;
        let player_b_position = self.get_player_b_position() as usize;

        if self.has_player_a_won() || self.has_player_b_won() {
            return false;
        }

        for combination in Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS.iter() {
            if player_a_position == combination.player_a_position {
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

const INVALID_INDEX: usize = usize::MAX;

#[derive(Copy, Clone)]
struct SimplifiedStateVariant {
    player_a_position: usize,
    player_b_options: usize,
    player_b_positions: [usize; 15],
    total_possible_states: u64,
}

impl SimplifiedStateVariant {
    const fn new(player_a_tile_id: usize, player_b_tile_ids: [usize; 15]) -> Self {
        let mut player_b_options = 0;
        let mut player_b_positions = [INVALID_INDEX; 15];

        let mut i = 0;
        while i < player_b_tile_ids.len() {
            let player_b_tile_id = player_b_tile_ids[i];
            if player_b_tile_id == INVALID_INDEX {
                break;
            }
            player_b_positions[i] = GameState4x4Binary3Bit::TILE_ID_TO_POSITION[player_b_tile_id];
            player_b_options += 1;
            i += 1;
        }

        let total_possible_states = player_b_options as u64 * 3u64.pow(2) * 5u64.pow(14);

        return Self {
            player_a_position: GameState4x4Binary3Bit::TILE_ID_TO_POSITION[player_a_tile_id],
            player_b_options,
            player_b_positions,
            total_possible_states,
        };
    }
}

impl GameState4x4Binary3Bit {
    const POSSIBLE_SIMPLIFIED_STATE_VARIANTS: [SimplifiedStateVariant; 3] = [
        SimplifiedStateVariant::new(
            0,
            [1, 2, 3, 5, 6, 7, 10, 11, 15, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX],
        ),
        SimplifiedStateVariant::new(
            1,
            [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        ),
        SimplifiedStateVariant::new(
            5,
            [0, 1, 2, 3, 6, 7, 10, 11, 15, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX],
        ),
    ];

    const SUMMED_POSSIBLE_SIMPLIFIED_STATE_OPTIONS: usize = Self::precompute_summed_possible_simplified_state_options();
    const fn precompute_summed_possible_simplified_state_options() -> usize {
        let mut summed_possible_simplified_state_options = 0;
        let mut variant_index = 0;
        while variant_index < Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS.len() {
            summed_possible_simplified_state_options += Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS[variant_index].player_b_options;
            variant_index += 1;
        }
        return summed_possible_simplified_state_options;
    }
}

impl ContinuousId for GameState4x4Binary3Bit {
    fn get_continuous_id_count() -> u64 {
        let mut continuous_id_count = 0;

        for variant in Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS.iter() {
            continuous_id_count += variant.total_possible_states;
        }

        return continuous_id_count;
    }

    fn get_continuous_id(&self) -> u64 {
        let player_a_position = self.get_player_a_position();
        let player_b_position = self.get_player_b_position();

        debug_assert!(self.is_simplified());


        let matching_variant_index = Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS.iter().position(|&x| x.player_a_position == player_a_position as usize)
            .expect("No variant matching player A position found, this can only happen for non simplified states");
        let matching_variant = &Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS[matching_variant_index];

        let mut variant_offset = 0;
        for i in 0..matching_variant_index {
            variant_offset += Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS[i].total_possible_states;
        }

        let player_b_position_index = matching_variant.player_b_positions.iter().position(|&x| x == player_b_position as usize)
            .expect("Player B position not found, this can only happen for non simplified states");

        let mut continuous_id = 0;

        let mut raw_value = self.0;
        for pos in 0..16 {
            let height = raw_value & 0x7;
            if pos == player_a_position || pos == player_b_position {
                continuous_id = continuous_id * 3 + height;
            } else {
                continuous_id = continuous_id * 5 + height;
            }
            raw_value >>= 3;
        }

        continuous_id = continuous_id * matching_variant.player_b_options as u64 + player_b_position_index as u64;
        continuous_id += variant_offset;

        return continuous_id;
    }

    fn from_continuous_id(mut continuous_id: u64) -> Self {
        let mut matching_variant_option = None;
        for variant in Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS.iter() {
            if continuous_id < variant.total_possible_states {
                matching_variant_option = Some(variant);
                break;
            }
            continuous_id -= variant.total_possible_states;
        }
        let matching_variant = matching_variant_option.
            expect("No matching variant found, this means that the continuous id is too high");

        let player_b_index = (continuous_id % matching_variant.player_b_options as u64) as usize;
        continuous_id /= matching_variant.player_b_options as u64;
        let player_a_position = matching_variant.player_a_position as u64;
        let player_b_position = matching_variant.player_b_positions[player_b_index] as u64;

        let mut raw_value = 0;
        for i in (0..16).rev() {
            let options = if i == player_a_position || i == player_b_position { 3 } else { 5 };
            let height = continuous_id % options as u64;
            continuous_id /= options as u64;
            raw_value = raw_value << 3 | height;
        }

        raw_value |= player_a_position << 48;
        raw_value |= player_b_position << 52;

        // Winning bits don't need to be checked, because the continuous mapping does not map to states where the players are on height 3

        return Self(raw_value);
    }
}


#[derive(Debug)]
struct HeightCount {
    height: u8,
    count: u8,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct TileHeightCombinationInput {
    player_a_height: u8,
    player_b_height: u8,
    height_4_tiles: u8,
    height_3_tiles: u8,
    height_2_tiles: u8,
    height_1_tiles: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct TileHeightCombination {
    player_a_height: u8,
    player_b_height: u8,
    height_4_tiles: u8,
    height_3_tiles: u8,
    height_2_tiles: u8,
    height_1_tiles: u8,
    possible_state_count: u64, // Including the player positions
    previous_summed_state_offset: u64, // Sum of all possible states before this one
}

impl TileHeightCombination {
    const fn new(player_a_height: u8, player_b_height: u8, height_4_tiles: u8, height_3_tiles: u8, height_2_tiles: u8, height_1_tiles: u8, previous_summed_state_offset: u64) -> Self {
        debug_assert!(player_a_height <= 2);
        debug_assert!(player_b_height <= 2);
        debug_assert!(height_4_tiles <= 14);
        debug_assert!(height_3_tiles <= 14);
        debug_assert!(height_2_tiles <= 14);
        debug_assert!(height_1_tiles <= 14);
        debug_assert!(height_4_tiles + height_3_tiles + height_2_tiles + height_1_tiles <= 14);


        let possible_state_count = get_binomial_coefficient(14, height_4_tiles as u64) *
            get_binomial_coefficient(14 - height_4_tiles as u64, height_3_tiles as u64) *
            get_binomial_coefficient(14 - height_4_tiles as u64 - height_3_tiles as u64, height_2_tiles as u64) *
            get_binomial_coefficient(14 - height_4_tiles as u64 - height_3_tiles as u64 - height_2_tiles as u64, height_1_tiles as u64) *
            GameState4x4Binary3Bit::SUMMED_POSSIBLE_SIMPLIFIED_STATE_OPTIONS as u64; // Player A and B positions


        return Self {
            player_a_height,
            player_b_height,
            height_4_tiles,
            height_3_tiles,
            height_2_tiles,
            height_1_tiles,
            possible_state_count,
            previous_summed_state_offset,
        };
    }

    const fn new_invalid() -> Self {
        return Self {
            player_a_height: u8::MAX,
            player_b_height: u8::MAX,
            height_4_tiles: u8::MAX,
            height_3_tiles: u8::MAX,
            height_2_tiles: u8::MAX,
            height_1_tiles: u8::MAX,
            possible_state_count: 0,
            previous_summed_state_offset: u64::MAX,
        };
    }

    const fn is_valid(&self) -> bool {
        return self.player_a_height != u8::MAX;
    }

    fn get_height_counts(self) -> [HeightCount; 4] {
        return [
            HeightCount { height: 4, count: self.height_4_tiles },
            HeightCount { height: 3, count: self.height_3_tiles },
            HeightCount { height: 2, count: self.height_2_tiles },
            HeightCount { height: 1, count: self.height_1_tiles }
        ];
    }
}


static COMBINATION_OFFSET_MAP: Lazy<HashMap<TileHeightCombinationInput, u64>> = Lazy::new(|| get_combination_offset_map());

fn get_combination_offset_map() -> HashMap<TileHeightCombinationInput, u64> {
    let mut combination_offset_map = HashMap::new();

    for block_count in 0..=64 {
        for combination in &GameState4x4Binary3Bit::TILE_HEIGHT_COMBINATIONS[block_count] {
            if !combination.is_valid() {
                break;
            }
            let input = TileHeightCombinationInput {
                player_a_height: combination.player_a_height,
                player_b_height: combination.player_b_height,
                height_4_tiles: combination.height_4_tiles,
                height_3_tiles: combination.height_3_tiles,
                height_2_tiles: combination.height_2_tiles,
                height_1_tiles: combination.height_1_tiles,
            };
            combination_offset_map.insert(input, combination.previous_summed_state_offset);
        }
    }

    return combination_offset_map;
}

impl GameState4x4Binary3Bit {
    const MAX_TILE_HEIGHT_COMBINATIONS_FOR_BLOCK_COUNT: [usize; 65] = Self::precompute_max_tile_height_combinations_for_block_count();
    const fn precompute_max_tile_height_combinations_for_block_count() -> [usize; 65] {
        let mut max_tile_height_combinations = [0; 65];

        let mut block_amount: usize = 0;
        while block_amount <= 64 {
            let mut possible_combinations = 0;
            let mut height_4_tiles = block_amount as isize / 4;
            while height_4_tiles >= 0 {
                let mut height_3_tiles = (block_amount as isize - height_4_tiles * 4) / 3;
                while height_3_tiles >= 0 {
                    let mut height_2_tiles = (block_amount as isize - height_4_tiles * 4 - height_3_tiles * 3) / 2;
                    while height_2_tiles >= 0 {
                        let height_1_tiles = block_amount as isize - height_4_tiles * 4 - height_3_tiles * 3 - height_2_tiles * 2;
                        if height_4_tiles + height_3_tiles + height_2_tiles + height_1_tiles <= 16 {
                            let height_0_tiles = 16 - height_4_tiles - height_3_tiles - height_2_tiles - height_1_tiles;
                            let tile_count = [height_0_tiles, height_1_tiles, height_2_tiles, height_3_tiles, height_4_tiles];

                            let mut player_a_height = 0;
                            while player_a_height <= 2 {
                                if tile_count[player_a_height as usize] <= 0 {
                                    player_a_height += 1;
                                    continue;
                                }

                                let mut player_b_height = 0;
                                while player_b_height <= 2 {
                                    if tile_count[player_b_height as usize] <= if player_b_height == player_a_height { 1 } else { 0 } {
                                        player_b_height += 1;
                                        continue;
                                    }

                                    possible_combinations += 1;

                                    player_b_height += 1;
                                }
                                player_a_height += 1;
                            }
                        }
                        height_2_tiles -= 1;
                    }
                    height_3_tiles -= 1;
                }
                height_4_tiles -= 1;
            }
            max_tile_height_combinations[block_amount] = possible_combinations;

            block_amount += 1;
        }

        return max_tile_height_combinations;
    }

    const MAX_TILE_HEIGHT_COMBINATIONS: usize = Self::precompute_max_tile_height_combinations();
    const fn precompute_max_tile_height_combinations() -> usize {
        let mut max_tile_height_combinations = 0;

        let mut block_amount: usize = 0;
        while block_amount <= 64 {
            if Self::MAX_TILE_HEIGHT_COMBINATIONS_FOR_BLOCK_COUNT[block_amount] > max_tile_height_combinations {
                max_tile_height_combinations = Self::MAX_TILE_HEIGHT_COMBINATIONS_FOR_BLOCK_COUNT[block_amount];
            }

            block_amount += 1;
        }

        return max_tile_height_combinations;
    }

    const TILE_HEIGHT_COMBINATIONS: [[TileHeightCombination; Self::MAX_TILE_HEIGHT_COMBINATIONS]; 65] = Self::precompute_tile_height_combinations();
    const fn precompute_tile_height_combinations() -> [[TileHeightCombination; Self::MAX_TILE_HEIGHT_COMBINATIONS]; 65] {
        let mut tile_height_combinations = [[TileHeightCombination::new_invalid(); Self::MAX_TILE_HEIGHT_COMBINATIONS]; 65];


        let mut block_amount: usize = 0;
        while block_amount <= 64 {
            let mut summed_state_offset = 0;
            let mut combinations_index = 0;
            let mut height_4_tiles = block_amount as isize / 4;
            while height_4_tiles >= 0 {
                let mut height_3_tiles = (block_amount as isize - height_4_tiles * 4) / 3;
                while height_3_tiles >= 0 {
                    let mut height_2_tiles = (block_amount as isize - height_4_tiles * 4 - height_3_tiles * 3) / 2;
                    while height_2_tiles >= 0 {
                        let height_1_tiles = block_amount as isize - height_4_tiles * 4 - height_3_tiles * 3 - height_2_tiles * 2;
                        if height_4_tiles + height_3_tiles + height_2_tiles + height_1_tiles <= 16 {
                            let height_0_tiles = 16 - height_4_tiles - height_3_tiles - height_2_tiles - height_1_tiles;
                            let tile_count = [height_0_tiles, height_1_tiles, height_2_tiles, height_3_tiles, height_4_tiles];

                            let mut player_a_height = 0;
                            while player_a_height <= 2 {
                                if tile_count[player_a_height as usize] <= 0 {
                                    player_a_height += 1;
                                    continue;
                                }

                                let mut player_b_height = 0;
                                while player_b_height <= 2 {
                                    if tile_count[player_b_height as usize] <= if player_b_height == player_a_height { 1 } else { 0 } {
                                        player_b_height += 1;
                                        continue;
                                    }

                                    tile_height_combinations[block_amount][combinations_index] = TileHeightCombination::new(
                                        player_a_height as u8,
                                        player_b_height as u8,
                                        (height_4_tiles - if player_a_height == 4 { 1 } else { 0 } - if player_b_height == 4 { 1 } else { 0 }) as u8,
                                        (height_3_tiles - if player_a_height == 3 { 1 } else { 0 } - if player_b_height == 3 { 1 } else { 0 }) as u8,
                                        (height_2_tiles - if player_a_height == 2 { 1 } else { 0 } - if player_b_height == 2 { 1 } else { 0 }) as u8,
                                        (height_1_tiles - if player_a_height == 1 { 1 } else { 0 } - if player_b_height == 1 { 1 } else { 0 }) as u8,
                                        summed_state_offset,
                                    );
                                    summed_state_offset += tile_height_combinations[block_amount][combinations_index].possible_state_count;
                                    combinations_index += 1;

                                    player_b_height += 1;
                                }
                                player_a_height += 1;
                            }
                        }
                        height_2_tiles -= 1;
                    }
                    height_3_tiles -= 1;
                }
                height_4_tiles -= 1;
            }

            block_amount += 1;
        }

        return tile_height_combinations;
    }

    fn get_possible_state_count_for_remaining(mut remaining_tiles: u64, remaining_counts: Vec<u64>) -> u64 {
        debug_assert!(remaining_tiles >= remaining_counts.iter().sum());

        let mut possible_state_count = 1;

        for remaining_count in remaining_counts {
            possible_state_count *= get_binomial_coefficient(remaining_tiles, remaining_count);
            remaining_tiles -= remaining_count;
        }

        return possible_state_count;
    }

    fn find_matching_combination(block_count: usize, continuous_id: u64) -> &'static TileHeightCombination {
        let tile_height_combinations = &Self::TILE_HEIGHT_COMBINATIONS[block_count];

        let mut low = 0;
        let mut high = Self::MAX_TILE_HEIGHT_COMBINATIONS_FOR_BLOCK_COUNT[block_count];

        while low < high {
            let mid = low + (high - low) / 2;
            let mid_combination = &tile_height_combinations[mid];
            if mid_combination.previous_summed_state_offset <= continuous_id {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        return if low == 0 {
            &tile_height_combinations[0]
        } else {
            &tile_height_combinations[low - 1]
        };
    }
}

impl ContinuousBlockId for GameState4x4Binary3Bit {
    fn get_block_count(&self) -> u64 {
        let mut block_count = 0;
        let mut data = self.0;
        for _ in 0..16 {
            block_count += data & 0x7;
            data >>= 3;
        }
        return block_count;
    }

    fn get_continuous_block_id_count(block_count: usize) -> u64 {
        if block_count > Self::TILE_HEIGHT_COMBINATIONS.len() {
            return 0;
        }
        let mut continuous_block_id_count = 0;

        for combination in &Self::TILE_HEIGHT_COMBINATIONS[block_count] {
            if !combination.is_valid() {
                break;
            }
            continuous_block_id_count += combination.possible_state_count;
        }

        return continuous_block_id_count;
    }

    fn get_continuous_block_id(&self) -> u64 {
        debug_assert!(self.is_simplified());

        let position_heights = self.get_position_heights();

        let player_a_position = self.get_player_a_position();
        let player_b_position = self.get_player_b_position();
        let player_a_height = position_heights[player_a_position as usize];
        let player_b_height = position_heights[player_b_position as usize];

        let mut block_count = 0;
        let mut height_counts: [u8; 5] = [0; 5];

        let mut available_positions: Vec<usize> = Vec::with_capacity(14);

        for (position, height) in position_heights.iter().enumerate() {
            block_count += height;
            if position == player_a_position as usize || position == player_b_position as usize {
                continue;
            }
            available_positions.push(position);
            height_counts[*height as usize] += 1;
        }

        let combination_id_offset = *COMBINATION_OFFSET_MAP.get(&TileHeightCombinationInput {
            player_a_height,
            player_b_height,
            height_4_tiles: height_counts[4],
            height_3_tiles: height_counts[3],
            height_2_tiles: height_counts[2],
            height_1_tiles: height_counts[1],
        }).expect("Combination not found, this should not be possible");

        let mut tile_id_offset = 0;
        for height in (1..=4).rev() {
            let height_count = height_counts[height];
            let mut tile_offset = 0;

            let mut remaining_counts = Vec::new();
            for remaining_height in 1..height {
                remaining_counts.push(height_counts[remaining_height] as u64);
            }
            let remaining_height_options = Self::get_possible_state_count_for_remaining(available_positions.len() as u64 - height_count as u64, remaining_counts);

            for height_count_offset in 0..height_count {
                while position_heights[available_positions[tile_offset]] as usize != height {
                    let current_height_options_if_tile_is_chosen = get_binomial_coefficient((available_positions.len() - tile_offset - 1) as u64, (height_count - height_count_offset - 1) as u64);
                    let possible_state_count_if_tile_is_chosen = current_height_options_if_tile_is_chosen * remaining_height_options;

                    tile_id_offset += possible_state_count_if_tile_is_chosen;
                    tile_offset += 1;
                }

                available_positions.remove(tile_offset);
            }
        }

        let mut continuous_id = combination_id_offset + tile_id_offset * Self::SUMMED_POSSIBLE_SIMPLIFIED_STATE_OPTIONS as u64;

        for variant in &Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS {
            if player_a_position as usize == variant.player_a_position {
                let player_b_position_index = variant.player_b_positions.iter().position(|&x| x == player_b_position as usize)
                    .expect("Player B position not found, this can only happen for non simplified states");
                continuous_id += player_b_position_index as u64;
                break;
            }
            continuous_id += variant.player_b_options as u64;
        }

        return continuous_id;
    }

    fn from_continuous_block_id(block_count: usize, mut continuous_id: u64) -> Self {
        let matching_combination = Self::find_matching_combination(block_count, continuous_id);
        continuous_id -= matching_combination.previous_summed_state_offset;

        // Found the correct combination

        let mut available_positions: Vec<usize> = (0..=15).collect();
        let mut position_heights = [0; 16];


        let mut option_index = continuous_id % Self::SUMMED_POSSIBLE_SIMPLIFIED_STATE_OPTIONS as u64;
        continuous_id /= Self::SUMMED_POSSIBLE_SIMPLIFIED_STATE_OPTIONS as u64;

        let mut player_a_position_option = None;
        let mut player_b_position_option = None;
        for variant in &Self::POSSIBLE_SIMPLIFIED_STATE_VARIANTS {
            if option_index < variant.player_b_options as u64 {
                player_a_position_option = Some(variant.player_a_position);
                player_b_position_option = Some(variant.player_b_positions[option_index as usize]);
                break;
            }
            option_index -= variant.player_b_options as u64;
        }
        let player_a_position = player_a_position_option.expect("No matching variant found, this should not be possible");
        let player_b_position = player_b_position_option.expect("No matching variant found, this should not be possible");


        available_positions.retain(|&x| x != player_a_position && x != player_b_position);
        position_heights[player_a_position] = matching_combination.player_a_height;
        position_heights[player_b_position] = matching_combination.player_b_height;


        let height_counts = matching_combination.get_height_counts();

        for (height_count_index, height_count) in height_counts.iter().enumerate() {
            let mut tile_offset = 0;

            let remaining_counts: Vec<u64> = height_counts.iter().skip(height_count_index + 1).map(|x| x.count as u64).collect();
            let remaining_height_options = Self::get_possible_state_count_for_remaining(available_positions.len() as u64 - height_count.count as u64, remaining_counts);

            for height_count_offset in 0..height_count.count {
                loop {
                    let current_height_options_if_tile_is_chosen = get_binomial_coefficient((available_positions.len() - tile_offset - 1) as u64, (height_count.count - height_count_offset - 1) as u64);
                    let possible_state_count_if_tile_is_chosen = current_height_options_if_tile_is_chosen * remaining_height_options;

                    if continuous_id < possible_state_count_if_tile_is_chosen {
                        break;
                    }

                    continuous_id -= possible_state_count_if_tile_is_chosen;
                    tile_offset += 1;
                }

                // Found the correct tile
                let position = available_positions.remove(tile_offset);
                position_heights[position] = height_count.height;
            }
        }

        let mut raw_value = 0;
        for height in position_heights.iter().rev() {
            raw_value = raw_value << 3 | *height as u64;
        }

        raw_value |= (player_a_position as u64) << 48;
        raw_value |= (player_b_position as u64) << 52;

        // Winning bits don't need to be checked, because the continuous mapping does not map to states where the players are on height 3
        return Self(raw_value);
    }
}