pub mod gs4x4_static_evaluation {
    /*
    12 10 9  8
    13 15 11 6
    14 3  7  5
    0  1  2  4
     */
    use crate::game_state::SantoriniState4x4;

    const TILE_ID_TO_POSITION: [usize; 16] = [0, 1, 2, 4, 14, 3, 7, 5, 13, 15, 11, 6, 12, 10, 9, 8];

    const DISTANCE_TO_STATIC_VALUATION: [f32; 4] = [5.0, 2.0, 1.0, 0.5];
    const HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION: [[f32; 5]; 3] = [
        [1.0, 1.5, -1.0, 0.0, -1.0], //Start height 0
        [1.0, 1.5, 2.0, 0.5, -1.0], //Start height 1
        [1.0, 1.5, 2.0, 3.0, -1.0], //Start height 2
    ];

    const POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION: [[[[f32; 5]; 3]; 16]; 16] =
        precompute_position_to_position_to_height_to_height_to_valuation();
    const fn precompute_position_to_position_to_height_to_height_to_valuation() -> [[[[f32; 5]; 3]; 16]; 16] {
        let mut position_to_position_to_height_to_height_to_valuation = [[[[0.0; 5]; 3]; 16]; 16];

        let mut i = 0;
        while i < 16 {
            let row_i = i / 4;
            let column_i = i % 4;
            let position_i = TILE_ID_TO_POSITION[i];
            let mut j = 0;
            while j < 16 {
                let row_j = j / 4;
                let column_j = j % 4;
                let position_j = TILE_ID_TO_POSITION[j];

                let row_distance = if row_i > row_j { row_i - row_j } else { row_j - row_i };
                let column_distance = if column_i > column_j { column_i - column_j } else { column_j - column_i };
                let distance = if row_distance > column_distance { row_distance } else { column_distance };

                let mut start_height = 0;
                while start_height <= 2 {
                    let mut neighbor_height = 0;
                    while neighbor_height <= 4 {
                        let height_valuation = HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION[start_height][neighbor_height];
                        let distance_valuation = DISTANCE_TO_STATIC_VALUATION[distance];
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


    // Positions are 0-15 if placed, or 16 if not placed
    pub fn get_child_evaluation(state: SantoriniState4x4) -> f32 {
        let worker_a_position = state.worker_a_position as usize;
        let worker_b_position = state.worker_b_position as usize;

        if worker_a_position < 16 && state.position_heights[worker_a_position] == 3 {
            return if state.player_a_turn { f32::MAX } else { f32::MIN };
        } else if worker_b_position < 16 && state.position_heights[worker_b_position] == 3 {
            return if state.player_a_turn { f32::MIN } else { f32::MAX };
        }

        let mut valuation = 0.0;

        // TODO: Consider whose turn it is
        // TODO: Consider unplaced workers

        for i in 0..16 {
            valuation += POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[worker_a_position][i][state.position_heights[worker_a_position] as usize][state.position_heights[i] as usize];
            valuation -= POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[worker_b_position][i][state.position_heights[worker_b_position] as usize][state.position_heights[i] as usize];
        }

        return valuation;
    }
}

pub mod gs5x5_static_evaluation {
    use crate::game_state::SantoriniState5x5;
    use crate::strategy::heuristics;

    const NO_NEIGHBOR: usize = heuristics::NO_NEIGHBOR;
    const TILE_TO_NEIGHBORS: [[usize; 8]; 25] = heuristics::TILE_TO_NEIGHBORS;


    const DISTANCE_TO_STATIC_VALUATION: [f32; 5] = [5.0, 2.0, 1.0, 0.5, 0.0];
    const HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION: [[f32; 5]; 3] = [
        [1.0, 1.5, -1.0, 0.0, -1.0], //Start height 0
        [1.0, 1.5, 2.0, 0.5, -1.0], //Start height 1
        [1.0, 1.5, 2.0, 3.0, -1.0], //Start height 2
    ];

    const TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION: [[[[f32; 5]; 3]; 25]; 25] =
        precompute_tile_to_tile_to_height_to_height_to_valuation();
    const fn precompute_tile_to_tile_to_height_to_height_to_valuation() -> [[[[f32; 5]; 3]; 25]; 25] {
        let mut tile_to_tile_to_height_to_height_to_valuation = [[[[0.0; 5]; 3]; 25]; 25];

        let mut worker_tile_id = 0;
        while worker_tile_id < 25 {
            let worker_tile_row = worker_tile_id / 5;
            let worker_tile_column = worker_tile_id % 5;
            let mut other_tile_id = 0;
            while other_tile_id < 25 {
                let other_tile_row = other_tile_id / 5;
                let other_tile_column = other_tile_id % 5;

                let row_distance = if worker_tile_row > other_tile_row { worker_tile_row - other_tile_row } else { other_tile_row - worker_tile_row };
                let column_distance = if worker_tile_column > other_tile_column { worker_tile_column - other_tile_column } else { other_tile_column - worker_tile_column };
                let distance = if row_distance > column_distance { row_distance } else { column_distance };

                let mut worker_height = 0;
                while worker_height <= 2 {
                    let mut other_height = 0;
                    while other_height <= 4 {
                        let height_valuation = HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION[worker_height][other_height];
                        let distance_valuation = DISTANCE_TO_STATIC_VALUATION[distance];
                        tile_to_tile_to_height_to_height_to_valuation[worker_tile_id][other_tile_id][worker_height][other_height] = height_valuation * distance_valuation;
                        other_height += 1;
                    }

                    worker_height += 1;
                }
                other_tile_id += 1;
            }
            worker_tile_id += 1;
        }

        return tile_to_tile_to_height_to_height_to_valuation;
    }

    // Positions are 0-24 if placed, or 25 if not placed
    // Undefined behavior if the state is already won, it is expected that the caller checks this before calling this function
    pub fn get_static_evaluation_old(tile_heights: [u8; 25], player_a_worker_tiles: [u8; 2], player_b_worker_tiles: [u8; 2], _player_a_turn: bool) -> f32 {
        if player_b_worker_tiles[0] == 25 {
            // TODO: Consider unplaced workers better
            // Unplaced workers
            return 0.0;
        }

        let workers_a_tile_and_height = [
            (player_a_worker_tiles[0] as usize, tile_heights[player_a_worker_tiles[0] as usize] as usize),
            (player_a_worker_tiles[1] as usize, tile_heights[player_a_worker_tiles[1] as usize] as usize),
        ];
        let workers_b_tile_and_height = [
            (player_b_worker_tiles[0] as usize, tile_heights[player_b_worker_tiles[0] as usize] as usize),
            (player_b_worker_tiles[1] as usize, tile_heights[player_b_worker_tiles[1] as usize] as usize),
        ];

        debug_assert!(workers_a_tile_and_height[0].1 != 3 && workers_a_tile_and_height[1].1 != 3);
        debug_assert!(workers_b_tile_and_height[0].1 != 3 && workers_b_tile_and_height[1].1 != 3);

        let mut valuation = 0.0;

        // TODO: Consider whose turn it is

        for i in 0..25 {
            let tile_height = tile_heights[i] as usize;
            for (a_worker_tile, a_worker_height) in &workers_a_tile_and_height {
                valuation += TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[*a_worker_tile][i][*a_worker_height][tile_height];
            }
            for (b_worker_tile, b_worker_height) in &workers_b_tile_and_height {
                valuation -= TILE_TO_TILE_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[*b_worker_tile][i][*b_worker_height][tile_height];
            }
        }

        return valuation;
    }


    const TILE_TO_TILE_TO_DISTANCE: [[usize; 25]; 25] = precompute_tile_to_tile_to_distance();
    const fn precompute_tile_to_tile_to_distance() -> [[usize; 25]; 25] {
        let mut tile_to_tile_to_distance = [[0; 25]; 25];

        let mut tile_1 = 0;
        while tile_1 < 25 {
            let row_1 = tile_1 / 5;
            let column_1 = tile_1 % 5;
            let mut tile_2 = 0;
            while tile_2 < 25 {
                let row_2 = tile_2 / 5;
                let column_2 = tile_2 % 5;

                let row_distance = if row_1 > row_2 { row_1 - row_2 } else { row_2 - row_1 };
                let column_distance = if column_1 > column_2 { column_1 - column_2 } else { column_2 - column_1 };
                let distance = if row_distance > column_distance { row_distance } else { column_distance };

                tile_to_tile_to_distance[tile_1][tile_2] = distance;
                tile_2 += 1;
            }
            tile_1 += 1;
        }

        return tile_to_tile_to_distance;
    }

    const ACTIVE_WORKER_HEIGHT_TO_NEIGHBOR_HEIGHT_TO_VALUATION: [[f32; 5]; 3] = [
        [0.0, 1.0, 1.0, 0.0, -2.0], //Start height 0
        [0.0, 1.0, 3.0, 0.0, -2.0], //Start height 1
        [0.0, 1.0, 4.0, f32::MAX, -1.0], //Start height 2
    ];

    const INACTIVE_WORKER_HEIGHT_TO_NEIGHBOR_HEIGHT_TO_VALUATION: [[f32; 5]; 3] = [
        [0.3, 0.4, 0.5, 0.0, 0.0], //Start height 0
        [0.3, 0.5, 1.0, 0.5, 0.0], //Start height 1
        [0.3, 0.5, 1.7, 10.0, 0.0], //Start height 2
    ];

    const CENTER_DISTANCE_VALUATIONS: [f32; 6] = [
        2.0, // Distance 0
        0.8, // Distance 1
        0.0, // Distance sqrt(2)
        0.0, // Distance 2
        0.0, // Distance sqrt(5)
        1.0, // Distance sqrt(8)
    ];
    const TILE_CENTER_VALUE: [f32; 25] = [
        CENTER_DISTANCE_VALUATIONS[5], CENTER_DISTANCE_VALUATIONS[4], CENTER_DISTANCE_VALUATIONS[3], CENTER_DISTANCE_VALUATIONS[4], CENTER_DISTANCE_VALUATIONS[5],
        CENTER_DISTANCE_VALUATIONS[4], CENTER_DISTANCE_VALUATIONS[2], CENTER_DISTANCE_VALUATIONS[1], CENTER_DISTANCE_VALUATIONS[2], CENTER_DISTANCE_VALUATIONS[4],
        CENTER_DISTANCE_VALUATIONS[3], CENTER_DISTANCE_VALUATIONS[1], CENTER_DISTANCE_VALUATIONS[0], CENTER_DISTANCE_VALUATIONS[1], CENTER_DISTANCE_VALUATIONS[3],
        CENTER_DISTANCE_VALUATIONS[4], CENTER_DISTANCE_VALUATIONS[2], CENTER_DISTANCE_VALUATIONS[1], CENTER_DISTANCE_VALUATIONS[2], CENTER_DISTANCE_VALUATIONS[4],
        CENTER_DISTANCE_VALUATIONS[5], CENTER_DISTANCE_VALUATIONS[4], CENTER_DISTANCE_VALUATIONS[3], CENTER_DISTANCE_VALUATIONS[4], CENTER_DISTANCE_VALUATIONS[5],
    ];


    // Positions are 0-24 if placed, or 25 if not placed
    // Undefined behavior if the state is already won, it is expected that the caller checks this before calling this function
    pub fn get_child_evaluation(state: SantoriniState5x5) -> f32 {
        if state.worker_b_tiles[0] == 25 {
            // TODO: Consider unplaced workers better
            // Unplaced workers
            return 0.0;
        }

        debug_assert!(state.tile_heights[state.worker_a_tiles[0] as usize] != 3 && state.tile_heights[state.worker_a_tiles[1] as usize] != 3);
        debug_assert!(state.tile_heights[state.worker_b_tiles[0] as usize] != 3 && state.tile_heights[state.worker_b_tiles[1] as usize] != 3);

        let mut valuation = 0.0;


        for a_worker_tile in &state.worker_a_tiles {
            let worker_tile = *a_worker_tile as usize;
            let worker_height = state.tile_heights[worker_tile] as usize;

            valuation += TILE_CENTER_VALUE[worker_tile];

            for neighbor_tile in &TILE_TO_NEIGHBORS[worker_tile] {
                if *neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[*neighbor_tile] as usize;
                if state.player_a_turn {
                    valuation += ACTIVE_WORKER_HEIGHT_TO_NEIGHBOR_HEIGHT_TO_VALUATION[worker_height][neighbor_tile_height];
                } else {
                    valuation += INACTIVE_WORKER_HEIGHT_TO_NEIGHBOR_HEIGHT_TO_VALUATION[worker_height][neighbor_tile_height];
                }
            }
        }

        for b_worker_tile in &state.worker_b_tiles {
            let worker_tile = *b_worker_tile as usize;
            let worker_height = state.tile_heights[worker_tile] as usize;

            valuation -= TILE_CENTER_VALUE[worker_tile];

            for neighbor_tile in &TILE_TO_NEIGHBORS[worker_tile] {
                if *neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[*neighbor_tile] as usize;
                if state.player_a_turn {
                    valuation -= INACTIVE_WORKER_HEIGHT_TO_NEIGHBOR_HEIGHT_TO_VALUATION[worker_height][neighbor_tile_height];
                } else {
                    valuation -= ACTIVE_WORKER_HEIGHT_TO_NEIGHBOR_HEIGHT_TO_VALUATION[worker_height][neighbor_tile_height];
                }
            }
        }

        return valuation;
    }
}