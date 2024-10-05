use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, SantoriniEval, SantoriniState5x5};
use crate::game_state::utils::child_evaluation::gs5x5_child_evaluation;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct GameState5x5Struct {
    pub tile_heights: [u8; 25],
    pub player_a_workers: [u8; 2],
    pub player_b_workers: [u8; 2],
    pub player_a_turn: bool,
    pub has_player_a_won: bool,
    pub has_player_b_won: bool,
}

impl fmt::Display for GameState5x5Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState5x5Struct {
    const WORKER_NOT_PLACED: u8 = 25;

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
}

impl GameState for GameState5x5Struct {
    type RawValue = GameState5x5Struct;
    type GenericGameState = GenericSantoriniGameState<5, 5, 2>;

    fn new(value: GameState5x5Struct) -> Self {
        return value;
    }

    fn raw_value(&self) -> GameState5x5Struct {
        return *self;
    }

    fn is_player_a_turn(&self) -> bool {
        return self.player_a_turn;
    }

    fn has_player_a_won(&self) -> bool {
        return self.has_player_a_won;
    }

    fn has_player_b_won(&self) -> bool {
        return self.has_player_b_won;
    }

    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<5, 5, 2>) -> Self {
        let mut tile_heights = [0; 25];

        for tile_id in 0..25 {
            tile_heights[tile_id] = generic_game_state.get_tile_height(tile_id);
        }

        let mut player_a_workers = [Self::WORKER_NOT_PLACED; 2];
        let mut player_b_workers = [Self::WORKER_NOT_PLACED; 2];
        for (i, &worker) in generic_game_state.player_a_workers.iter().flatten().enumerate() {
            player_a_workers[i] = worker;
        }
        for (i, &worker) in generic_game_state.player_b_workers.iter().flatten().enumerate() {
            player_b_workers[i] = worker;
        }

        return Self {
            tile_heights,
            player_a_workers,
            player_b_workers,
            player_a_turn: generic_game_state.player_a_turn,
            has_player_a_won: generic_game_state.has_player_a_won(),
            has_player_b_won: generic_game_state.has_player_b_won(),
        };
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<5, 5, 2> {
        let mut generic_tile_heights = [[0; 5]; 5];
        for i in 0..25 {
            generic_tile_heights[i / 5][i % 5] = self.tile_heights[i];
        }

        let generic_player_a_workers = if self.player_a_workers[0] == Self::WORKER_NOT_PLACED {
            None
        } else {
            Some(self.player_a_workers)
        };
        let generic_player_b_workers = if self.player_b_workers[0] == Self::WORKER_NOT_PLACED {
            None
        } else {
            Some(self.player_b_workers)
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


        let moving_player_workers = if self.player_a_turn { self.player_a_workers } else { self.player_b_workers };

        let mut tile_has_worker = [false; 25];
        for worker_tile in self.player_a_workers.iter().chain(self.player_b_workers.iter()) {
            if *worker_tile != Self::WORKER_NOT_PLACED {
                tile_has_worker[*worker_tile as usize] = true;
            }
        }


        if moving_player_workers[0] == Self::WORKER_NOT_PLACED {
            // Workers are not placed yet, so the next states are all possible worker placements
            for worker_1_tile_id in 0..25 {
                if tile_has_worker[worker_1_tile_id] {
                    continue;
                }
                for worker_2_tile_id in (worker_1_tile_id + 1)..25 {
                    if tile_has_worker[worker_2_tile_id] {
                        continue;
                    }

                    let new_workers = [worker_1_tile_id as u8, worker_2_tile_id as u8];
                    possible_next_states.push(Self {
                        tile_heights: self.tile_heights,
                        player_a_workers: if self.player_a_turn { new_workers } else { self.player_a_workers },
                        player_b_workers: if self.player_a_turn { self.player_b_workers } else { new_workers },
                        player_a_turn: !self.player_a_turn,
                        has_player_a_won: false,
                        has_player_b_won: false,
                    });
                }
            }

            return;
        }

        // All workers are placed, so the next states are all possible worker moves
        for worker_index in 0..2 {
            let worker_tile = moving_player_workers[worker_index] as usize;
            let worker_height = self.tile_heights[worker_tile];
            let max_movement_height = match worker_height {
                0 => 1,
                1 => 2,
                2 => 3,
                _ => panic!("Can't get children for a game state that is already won")
            };

            for movement_tile in Self::TILE_TO_NEIGHBORS[worker_tile] {
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
                    if tile_has_worker[build_tile] && build_tile != worker_tile {
                        continue;
                    }
                    let build_height = self.tile_heights[build_tile];
                    if build_height >= 4 {
                        continue;
                    }

                    let mut new_position_heights = self.tile_heights;
                    new_position_heights[build_tile] += 1;

                    let mut new_moving_player_workers = moving_player_workers;
                    new_moving_player_workers[worker_index] = movement_tile as u8;

                    if self.player_a_turn {
                        possible_next_states.push(Self {
                            tile_heights: new_position_heights,
                            player_a_workers: new_moving_player_workers,
                            player_b_workers: self.player_b_workers,
                            player_a_turn: false,
                            has_player_a_won: movement_height == 3,
                            has_player_b_won: false,
                        });
                    } else {
                        possible_next_states.push(Self {
                            tile_heights: new_position_heights,
                            player_a_workers: self.player_a_workers,
                            player_b_workers: new_moving_player_workers,
                            player_a_turn: true,
                            has_player_a_won: false,
                            has_player_b_won: movement_height == 3,
                        });
                    }
                }
            }
        }
    }
}


impl SantoriniEval for GameState5x5Struct {
    type SantoriniState = SantoriniState5x5;

    fn get_santorini_state(&self) -> Self::SantoriniState {
        return SantoriniState5x5 {
            tile_heights: self.tile_heights,
            worker_a_tiles: self.player_a_workers,
            worker_b_tiles: self.player_b_workers,
            player_a_turn: self.player_a_turn,
        };
    }

    fn get_child_evaluation(&self) -> f32 {
        if self.has_player_a_won {
            return f32::INFINITY;
        } else if self.has_player_b_won {
            return f32::NEG_INFINITY;
        }

        return gs5x5_child_evaluation::get_child_evaluation(self.get_santorini_state());
    }
}