use std::fmt;
use std::fmt::Formatter;
use crate::game_state::generic_game_state::GenericGameState;
use crate::game_state::utils::precompute_position_to_tile_id::precompute_position_to_tile_id;

/*
Bits 0-47: 3 bits per tile, 16 tiles
Bits 48-51: Player A position
Bits 52-55: Player B position
Bits 56-61: Unused
Bit 62: Player B has won (because they have reached height 3)
Bit 63: Player A has won (because they have reached height 3)

For each tile:
- Bits 0-2: Height (0-4)
 */
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Binary3BitGameState(u64);

impl fmt::Display for Binary3BitGameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl Binary3BitGameState {
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

    fn get_player_a_position(self) -> u64 {
        return (self.0 >> 48) & 0xF;
    }

    fn get_player_b_position(self) -> u64 {
        return (self.0 >> 52) & 0xF;
    }

    fn get_position_heights(self) -> [u8; 16] {
        let mut position_heights = [0; 16];

        let mut data = self.0;
        for i in 0..16 {
            position_heights[i] = (data & 0x7) as u8;
            data >>= 3;
        }
        return position_heights;
    }

    pub fn new(binary_game_state: u64) -> Self {
        return Self(binary_game_state);
    }

    pub fn raw_value(self) -> u64 {
        self.0
    }

    pub fn has_player_a_won(self) -> bool {
        return self.0 & (1 << 63) != 0;
    }

    pub fn has_player_b_won(self) -> bool {
        return self.0 & (1 << 62) != 0;
    }

    pub fn from_generic_game_state(generic_game_state: &GenericGameState) -> Self {
        let mut binary_game_state = 0;
        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            let height = generic_game_state.tile_heights[i];
            binary_game_state |= (height as u64) << (position * 3);
        }
        let player_a_position = Self::TILE_ID_TO_POSITION[generic_game_state.player_a_tile as usize] as u64;
        let player_b_position = Self::TILE_ID_TO_POSITION[generic_game_state.player_b_tile as usize] as u64;
        binary_game_state |= player_a_position << 48;
        binary_game_state |= player_b_position << 52;

        if generic_game_state.tile_heights[generic_game_state.player_a_tile as usize] == 3 {
            binary_game_state |= 1 << 63;
        }
        if generic_game_state.tile_heights[generic_game_state.player_b_tile as usize] == 3 {
            binary_game_state |= 1 << 62;
        }

        return Self(binary_game_state);
    }

    pub fn to_generic_game_state(self) -> GenericGameState {
        let position_heights = self.get_position_heights();
        let mut tile_heights = [0; 16];
        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            tile_heights[i] = position_heights[position];
        }
        // Convert position to tile
        let player_a_tile = Self::POSITION_TO_TILE_ID[self.get_player_a_position() as usize] as u8;
        let player_b_tile = Self::POSITION_TO_TILE_ID[self.get_player_b_position() as usize] as u8;
        return GenericGameState::new(player_a_tile, player_b_tile, tile_heights).expect("Invalid game state");
    }

    pub fn get_children_states(self) -> Vec<Self> {
        let mut possible_next_states = Vec::new();

        let player_a_position = self.get_player_a_position() as usize;
        let player_b_position = self.get_player_b_position() as usize;

        let player_a_height = (self.0 >> (player_a_position * 3)) & 0x7;
        let max_movement_height = match player_a_height {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => panic!("Can't get children for a game state that is already won")
        };

        for movement_position in Self::POSITION_TO_NEIGHBORS[player_a_position] {
            if movement_position == Self::NO_NEIGHBOR {
                continue;
            }
            if movement_position == player_b_position {
                continue;
            }
            let movement_height = (self.0 >> (movement_position * 3)) & 0x7;
            if movement_height > max_movement_height {
                continue;
            }

            for build_position in Self::POSITION_TO_NEIGHBORS[movement_position] {
                if build_position == Self::NO_NEIGHBOR {
                    continue;
                }
                if build_position == player_b_position {
                    continue;
                }
                let build_height = (self.0 >> (build_position * 3)) & 0x7;
                if build_height >= 4 {
                    continue;
                }

                let mut new_state = self.0;
                new_state &= !(0xF << 48);
                new_state |= (movement_position as u64) << 48;
                new_state += 1 << (build_position * 3);
                if movement_height == 3 {
                    new_state |= 1 << 63;
                }

                possible_next_states.push(Self(new_state));
            }
        }

        return possible_next_states;
    }

    pub fn get_flipped_state(self) -> Self {
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
        return Self(flipped_state);
    }
}


impl Binary3BitGameState {
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

    pub fn static_evaluation(self) -> f32 {
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

impl Binary3BitGameState {
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
        const ROTATION_MAP: [usize; 16] = [3,7,11,15,2,6,10,14,1,5,9,13,0,4,8,12];
        let mut i = 0;
        while i < ccw_90_rotations {
            new_tile_id = ROTATION_MAP[new_tile_id];
            i += 1;
        }

        return new_tile_id
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
                    0|1|4|5 => 0,
                    2|3|6|7 => 3,
                    10|11|14|15 => 2,
                    8|9|12|13 => 1,
                    _ => panic!("Invalid tile id")
                };

                let player_a_tile_rotated = Self::get_rotated_tile_id(player_a_tile, ccw_rotations);
                let diagonal_mirroring = if player_a_tile_rotated == 4 {
                    true
                } else {
                    let player_b_tile_rotated = Self::get_rotated_tile_id(player_b_tile, ccw_rotations);
                    match player_b_tile_rotated {
                        4|8|9|12|13|14 => true,
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

    pub fn get_symmetric_simplified_state(&self) -> Self {
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

        let new_state = new_height_information | (new_player_a_position << 48) | (new_player_b_position << 52) | (status_information << 56);

        return Self(new_state);
    }
}