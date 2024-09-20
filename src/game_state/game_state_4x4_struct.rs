use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, MinimaxReady};
use crate::game_state::game_state_5x5_struct::GameState5x5Struct;

use crate::game_state::utils::precompute_position_to_tile_id::precompute_position_to_tile_id;
use crate::game_state::utils::static_evaluation::gs4x4;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use crate::minimax::minimax_cache::MinimaxCache;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct GameState4x4Struct {
    tile_heights: [u8; 16],
    player_a_worker: u8,
    player_b_worker: u8,
    player_a_turn: bool,
}

impl fmt::Display for GameState4x4Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState4x4Struct {
    const WORKER_NOT_PLACED: u8 = 16;

    const NO_NEIGHBOR: usize = usize::MAX;
    const TILE_TO_NEIGHBORS: [[usize; 8]; 16] = Self::precompute_tile_to_neighbors();
    const fn precompute_tile_to_neighbors() -> [[usize; 8]; 16] {
        let mut position_to_neighbors = [[Self::NO_NEIGHBOR; 8]; 16];

        let mut row: isize = 0;
        while row < 4 {
            let mut column = 0;
            while column < 4 {
                let tile_id = (row * 4 + column) as usize;
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
                        let neighbor_tile_id = (neighbor_row * 4 + neighbor_column) as usize;
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
}

impl GameState for GameState4x4Struct {
    type RawValue = GameState4x4Struct;
    type GenericGameState = GenericSantoriniGameState<4, 4, 1>;

    fn new(value: GameState4x4Struct) -> Self {
        return value;
    }

    fn raw_value(&self) -> GameState4x4Struct {
        return *self;
    }

    fn is_player_a_turn(&self) -> bool {
        return self.player_a_turn;
    }

    fn has_player_a_won(&self) -> bool {
        if self.player_a_worker == Self::WORKER_NOT_PLACED {
            return false;
        }
        return self.tile_heights[self.player_a_worker as usize] == 3;
    }

    fn has_player_b_won(&self) -> bool {
        if self.player_b_worker == Self::WORKER_NOT_PLACED {
            return false;
        }
        return self.tile_heights[self.player_b_worker as usize] == 3;
    }


    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<4, 4, 1>) -> Self {
        let mut tile_heights = [0; 16];

        for tile_id in 0..16 {
            tile_heights[tile_id] = generic_game_state.get_tile_height(tile_id);
        }

        let player_a_worker = if let Some([worker]) = generic_game_state.player_a_workers {
            worker
        } else {
            Self::WORKER_NOT_PLACED
        };
        let player_b_worker = if let Some([worker]) = generic_game_state.player_b_workers {
            worker
        } else {
            Self::WORKER_NOT_PLACED
        };

        return Self {
            tile_heights,
            player_a_worker,
            player_b_worker,
            player_a_turn: generic_game_state.player_a_turn,
        };
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<4, 4, 1> {
        let mut generic_tile_heights = [[0; 4]; 4];
        for i in 0..16 {
            generic_tile_heights[i / 4][i % 4] = self.tile_heights[i];
        }

        let generic_player_a_workers = if self.player_a_worker == Self::WORKER_NOT_PLACED {
            None
        } else {
            Some([self.player_a_worker])
        };
        let generic_player_b_workers = if self.player_b_worker == Self::WORKER_NOT_PLACED {
            None
        } else {
            Some([self.player_b_worker])
        };

        return GenericSantoriniGameState::<4, 4, 1>::new(generic_player_a_workers, generic_player_b_workers, generic_tile_heights, self.player_a_turn)
            .expect("Invalid game state");
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


        let moving_player_worker = if self.player_a_turn { self.player_a_worker } else { self.player_b_worker };
        let other_player_worker = if self.player_a_turn { self.player_b_worker } else { self.player_a_worker };

        let mut tile_has_worker = [false; 25];
        for worker_tile in [moving_player_worker, other_player_worker].iter() {
            if *worker_tile != Self::WORKER_NOT_PLACED {
                tile_has_worker[*worker_tile as usize] = true;
            }
        }


        if moving_player_worker == Self::WORKER_NOT_PLACED {
            // Worker is not placed yet, so the next states are all possible worker placements
            for worker_tile_id in 0..16 {
                if tile_has_worker[worker_tile_id] {
                    continue;
                }

                possible_next_states.push(Self {
                    tile_heights: self.tile_heights,
                    player_a_worker: if self.player_a_turn { worker_tile_id as u8 } else { self.player_a_worker },
                    player_b_worker: if self.player_a_turn { self.player_b_worker } else { worker_tile_id as u8 },
                    player_a_turn: !self.player_a_turn,
                });
            }

            return;
        }

        // All workers are placed, so the next states are all possible worker moves
        let worker_height = self.tile_heights[moving_player_worker as usize];
        let max_movement_height = match worker_height {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => panic!("Can't get children for a game state that is already won")
        };

        for movement_tile in Self::TILE_TO_NEIGHBORS[moving_player_worker as usize] {
            if movement_tile == Self::NO_NEIGHBOR {
                break;
            }
            if tile_has_worker[movement_tile] {
                continue;
            }
            let movement_height = self.tile_heights[movement_tile];
            if movement_height > max_movement_height {
                continue;
            }

            for build_tile in Self::TILE_TO_NEIGHBORS[movement_tile] {
                if build_tile == Self::NO_NEIGHBOR {
                    break;
                }
                if tile_has_worker[build_tile] && build_tile != moving_player_worker as usize {
                    continue;
                }
                let build_height = self.tile_heights[build_tile];
                if build_height >= 4 {
                    continue;
                }

                let mut new_position_heights = self.tile_heights;
                new_position_heights[build_tile] += 1;

                possible_next_states.push(Self {
                    tile_heights: new_position_heights,
                    player_a_worker: if self.player_a_turn { movement_tile as u8 } else { self.player_a_worker },
                    player_b_worker: if self.player_a_turn { self.player_b_worker } else { movement_tile as u8 },
                    player_a_turn: !self.player_a_turn,
                });
            }
        }
    }
}

impl MinimaxReady for GameState4x4Struct {
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
        return gs4x4::get_static_evaluation(self.tile_heights, self.player_a_worker, self.player_b_worker, self.player_a_turn);
    }
}