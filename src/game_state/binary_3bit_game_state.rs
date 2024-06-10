use std::fmt;
use std::fmt::Formatter;
use crate::game_state::generic_game_state::GenericGameState;
use crate::game_state::utils::precompute_position_to_tile_id::precompute_position_to_tile_id;

/*
Bits 0-47: 3 bits per tile, 16 tiles
Bits 48-51: Player A position
Bits 52-55: Player B position

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
                        if neighbor_column < 0 || neighbor_column >= 4 || neighbor_row == row && neighbor_column == column {
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

    pub fn new(value: u64) -> Binary3BitGameState {
        Binary3BitGameState(value)
    }

    pub fn has_player_a_won(self) -> bool {
        let player_a_position = self.get_player_a_position() as usize;
        let player_a_height = (self.0 >> (player_a_position * 3)) & 0x7;
        return player_a_height == 3;
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

    pub fn get_possible_next_states(self) -> Vec<Binary3BitGameState> {
        let mut possible_next_states = Vec::new();

        let player_a_position = self.get_player_a_position() as usize;
        let player_b_position = self.get_player_b_position() as usize;

        let player_a_height = (self.0 >> (player_a_position * 3)) & 0x7;
        let max_movement_height = match player_a_height {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => return possible_next_states, // Player A has already won
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

                possible_next_states.push(Binary3BitGameState(new_state));
            }
        }

        return possible_next_states;
    }

    pub fn get_flipped_state(self) -> Binary3BitGameState {
        let mut flipped_state = self.0;
        let player_a_position = self.get_player_a_position();
        let player_b_position = self.get_player_b_position();

        // Clear the player positions
        flipped_state &= !(0xFF << 48);
        flipped_state |= player_a_position << 52;
        flipped_state |= player_b_position << 48;
        return Binary3BitGameState(flipped_state);
    }
}


impl Binary3BitGameState {
    const DISTANCE_TO_STATIC_VALUATION: [f32; 4] = [5.0, 2.0, 1.0, 0.5];
    const HEIGHT_TO_STATIC_VALUATION: [f32; 5] = [1.0, 1.5, 2.0, 3.0, -1.0];

    const POSITION_TO_POSITION_TO_HEIGHT_TO_VALUATION: [[[f32; 5]; 16]; 16] =
        Self::precompute_position_to_position_to_height_to_valuation();
    const fn precompute_position_to_position_to_height_to_valuation() -> [[[f32; 5]; 16]; 16] {
        let mut position_to_position_to_height_to_valuation = [[[0.0; 5]; 16]; 16];

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

                let mut height = 0;
                while height <= 4 {
                    let height_valuation = Self::HEIGHT_TO_STATIC_VALUATION[height];
                    let distance_valuation = Self::DISTANCE_TO_STATIC_VALUATION[distance];
                    position_to_position_to_height_to_valuation[position_i][position_j][height] = height_valuation * distance_valuation;
                    height += 1;
                }
                j += 1;
            }
            i += 1;
        }

        return position_to_position_to_height_to_valuation;
    }

    pub fn static_evaluation(self) -> f32 {
        let player_a_position = self.get_player_a_position() as usize;
        let player_b_position = self.get_player_b_position() as usize;
        let position_heights = self.get_position_heights();
        let mut valuation = 0.0;

        for i in 0..16 {
            valuation += Self::POSITION_TO_POSITION_TO_HEIGHT_TO_VALUATION[player_a_position][i][position_heights[i] as usize];
            valuation -= Self::POSITION_TO_POSITION_TO_HEIGHT_TO_VALUATION[player_b_position][i][position_heights[i] as usize];
        }
        return valuation;
    }
}

impl Binary3BitGameState {
    pub fn symmetric_transpose(&self) -> Self {
        let mut new_state;

        // TODO Also handle axis symmetry and combination of both

        // Rotation
        let height_information = self.0 & 0xFFFFFFFFFFFF;
        let player_a_position = self.get_player_a_position();
        let player_b_position = self.get_player_b_position();

        let rotations = player_a_position / 4;

        // Clockwise rotation
        match rotations {
            1 => {
                new_state = ((height_information & 0xFFF) << 36) | (height_information >> 12);
                new_state |= (player_a_position - 4 * rotations) << 48;
                new_state |= (player_b_position.wrapping_sub(4 * rotations) & 0xF) << 52;
            },
            2 => {
                new_state = ((height_information & 0xFFFFFF) << 24) | (height_information >> 24);
                new_state |= (player_a_position - 4 * rotations) << 48;
                new_state |= (player_b_position.wrapping_sub(4 * rotations) & 0xF) << 52;
            },
            3 => {
                new_state = ((height_information & 0xFFFFFFFFF) << 12) | (height_information >> 36);
                new_state |= (player_a_position - 4 * rotations) << 48;
                new_state |= (player_b_position.wrapping_sub(4 * rotations) & 0xF) << 52;
            },
            _ => {new_state = self.0;},
        }

        return Self::new(new_state);
    }
}