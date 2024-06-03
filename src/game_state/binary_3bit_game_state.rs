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
#[derive(Copy, Clone)]
pub(crate) struct Binary3BitGameState(u64);

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

    pub fn new(value: u64) -> Binary3BitGameState {
        Binary3BitGameState(value)
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
        let mut tile_heights = [0; 16];
        for i in 0..16 {
            let position = Self::TILE_ID_TO_POSITION[i];
            tile_heights[i] = ((self.0 >> (position * 3)) & 0x7) as u8;
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
            _ => 3,
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