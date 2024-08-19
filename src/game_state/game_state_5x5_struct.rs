
use std::fmt;
use std::fmt::Formatter;
use crate::game_state::{GameState, MinimaxReady};
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use crate::minimax::minimax_cache::MinimaxCache;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct GameState5x5Struct {
    pub tile_heights: [u8; 25],
    pub player_a_workers: [u8; 2],
    pub player_b_workers: [u8; 2],
    pub player_a_turn: bool,
}

impl fmt::Display for GameState5x5Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl fmt::Debug for GameState5x5Struct {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return self.to_generic_game_state().fmt(f);
    }
}

impl GameState5x5Struct {
    const WORKER_NOT_PLACED: u8 = u8::MAX;

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
        return self.player_a_workers.iter().any(|&x| x != GameState5x5Struct::WORKER_NOT_PLACED && self.tile_heights[x as usize] == 3);
    }

    fn has_player_b_won(&self) -> bool {
        return self.player_b_workers.iter().any(|&x| x != GameState5x5Struct::WORKER_NOT_PLACED && self.tile_heights[x as usize] == 3);
    }

    fn from_generic_game_state(generic_game_state: &GenericSantoriniGameState<5, 5, 2>) -> Self {
        let mut tile_heights = [0; 25];

        for tile_id in 0..25 {
            tile_heights[tile_id] = generic_game_state.get_tile_height(tile_id);
        }

        let mut player_a_workers = [Self::WORKER_NOT_PLACED; 2];
        let mut player_b_workers = [Self::WORKER_NOT_PLACED; 2];
        for (i, &worker) in generic_game_state.player_a_workers.iter().enumerate() {
            if worker != GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED {
                player_a_workers[i] = worker;
            }
        }
        for (i, &worker) in generic_game_state.player_b_workers.iter().enumerate() {
            if worker != GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED {
                player_b_workers[i] = worker;
            }
        }

        return Self {
            tile_heights,
            player_a_workers,
            player_b_workers,
            player_a_turn: generic_game_state.player_a_turn,
        };
    }

    fn to_generic_game_state(&self) -> GenericSantoriniGameState<5, 5, 2> {
        let mut tile_heights = [[0; 5]; 5];
        for row in 0..5 {
            for column in 0..5 {
                tile_heights[row][column] = self.tile_heights[row * 5 + column];
            }
        }

        let mut player_a_workers = [GenericSantoriniGameState::<5,5,2>::WORKER_NOT_PLACED; 2];
        let mut player_b_workers = [GenericSantoriniGameState::<5,5,2>::WORKER_NOT_PLACED; 2];

        for (i, &worker) in self.player_a_workers.iter().enumerate() {
            if worker != Self::WORKER_NOT_PLACED {
                player_a_workers[i] = worker;
            }
        }
        for (i, &worker) in self.player_b_workers.iter().enumerate() {
            if worker != Self::WORKER_NOT_PLACED {
                player_b_workers[i] = worker;
            }
        }

        return GenericSantoriniGameState::<5, 5, 2>::new(player_a_workers, player_b_workers, tile_heights, self.player_a_turn)
            .expect("Invalid game state");
    }

    fn get_children_states(&self) -> Vec<Self> {
        return self.get_children_states_reuse_vec(Vec::with_capacity(64));
    }

    fn get_children_states_reuse_vec(&self, vec: Vec<Self>) -> Vec<Self> {
        debug_assert!(!self.has_player_a_won());
        debug_assert!(!self.has_player_b_won());

        let mut possible_next_states = vec;
        possible_next_states.clear();


        let moving_player_workers = if self.player_a_turn { self.player_a_workers } else { self.player_b_workers };
        let other_player_workers = if self.player_a_turn { self.player_b_workers } else { self.player_a_workers };

        let mut tile_has_worker = [false; 25];
        for worker_tile in moving_player_workers.iter().chain(other_player_workers.iter()) {
            if *worker_tile != Self::WORKER_NOT_PLACED {
                tile_has_worker[*worker_tile as usize] = true;
            }
        }

        let worker_index_to_place_option = moving_player_workers.iter().position(|&x| x == Self::WORKER_NOT_PLACED);

        if let Some(worker_index_to_place) = worker_index_to_place_option {
            // Not all workers are placed yet, so the next states are all possible worker placements
            for tile_id in 0..25 {
                if !tile_has_worker[tile_id] {
                    let mut new_workers = moving_player_workers;
                    new_workers[worker_index_to_place] = tile_id as u8;
                    possible_next_states.push(Self {
                        tile_heights: self.tile_heights,
                        player_a_workers: if self.player_a_turn { new_workers } else { self.player_a_workers },
                        player_b_workers: if self.player_a_turn { self.player_b_workers } else { new_workers },
                        player_a_turn: !self.player_a_turn,
                    });
                }
            }

            return possible_next_states;
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

                let mut new_moving_player_workers = moving_player_workers;
                new_moving_player_workers[worker_index] = movement_tile as u8;

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

                    possible_next_states.push(Self {
                        tile_heights: new_position_heights,
                        player_a_workers: if self.player_a_turn { new_moving_player_workers } else { self.player_a_workers },
                        player_b_workers: if self.player_a_turn { self.player_b_workers } else { new_moving_player_workers },
                        player_a_turn: !self.player_a_turn,
                    });
                }
            }
        }

        return possible_next_states;
    }

    fn get_flipped_state(&self) -> Self {
        return Self {
            tile_heights: self.tile_heights,
            player_a_workers: self.player_b_workers,
            player_b_workers: self.player_a_workers,
            player_a_turn: !self.player_a_turn,
        };
    }
}


impl GameState5x5Struct {
    const DISTANCE_TO_STATIC_VALUATION: [f32; 5] = [5.0, 2.0, 1.0, 0.5, 0.0];
    const HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION: [[f32; 5]; 3] = [
        [1.0, 1.5, -1.0, 0.0, -1.0], //Start height 0
        [1.0, 1.5, 2.0, 0.5, -1.0], //Start height 1
        [1.0, 1.5, 2.0, 3.0, -1.0], //Start height 2
    ];

    const TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION: [[[[f32; 5]; 3]; 25]; 25] =
        Self::precompute_tile_to_tile_to_height_to_height_to_valuation();
    const fn precompute_tile_to_tile_to_height_to_height_to_valuation() -> [[[[f32; 5]; 3]; 25]; 25] {
        let mut tile_to_tile_to_height_to_height_to_valuation = [[[[0.0; 5]; 3]; 25]; 25];

        let mut i = 0;
        while i < 25 {
            let row_i = i / 5;
            let column_i = i % 5;
            let mut j = 0;
            while j < 25 {
                let row_j = j / 5;
                let column_j = j % 5;

                let row_distance = if row_i > row_j { row_i - row_j } else { row_j - row_i };
                let column_distance = if column_i > column_j { column_i - column_j } else { column_j - column_i };
                let distance = if row_distance > column_distance { row_distance } else { column_distance };

                let mut start_height = 0;
                while start_height <= 2 {
                    let mut neighbor_height = 0;
                    while neighbor_height <= 4 {
                        let height_valuation = Self::HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION[start_height][neighbor_height];
                        let distance_valuation = Self::DISTANCE_TO_STATIC_VALUATION[distance];
                        tile_to_tile_to_height_to_height_to_valuation[i][j][start_height][neighbor_height] = height_valuation * distance_valuation;
                        neighbor_height += 1;
                    }

                    start_height += 1;
                }
                j += 1;
            }
            i += 1;
        }

        return tile_to_tile_to_height_to_height_to_valuation;
    }
}

impl MinimaxReady for GameState5x5Struct {
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
        if self.has_player_a_won() {
            return f32::MAX;
        } else if self.has_player_b_won() {
            return f32::MIN;
        }

        let player_a_workers = self.player_a_workers.iter().map(|&x| x as usize).collect::<Vec<usize>>();
        let player_b_workers = self.player_b_workers.iter().map(|&x| x as usize).collect::<Vec<usize>>();

        let mut valuation = 0.0;

        // TODO: Consider whose turn it is

        for i in 0..25 {
            for player_a_worker in &player_a_workers {
                valuation += Self::TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[*player_a_worker][i][self.tile_heights[*player_a_worker] as usize][self.tile_heights[i] as usize];
            }
            for player_b_worker in &player_b_workers {
                valuation -= Self::TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[*player_b_worker][i][self.tile_heights[*player_b_worker] as usize][self.tile_heights[i] as usize];
            }
        }

        return valuation;
    }
}