use crate::game_state::{GameState, SantoriniEval, SantoriniState5x5};
use crate::strategy::heuristics::{NO_NEIGHBOR, TILE_TO_NEIGHBORS};


#[derive(Copy, Clone, Debug)]
pub struct DynamicHeuristicParams {
    pub active_worker_height_to_neighbor_height_to_valuation: [[f32; 5]; 3],
    pub active_worker_height_to_valuation: [f32; 3],
    pub inactive_worker_height_to_neighbor_height_to_valuation: [[f32; 5]; 3],
    pub inactive_worker_height_to_valuation: [f32; 3],
    pub center_distance_valuations: [f32; 6],
}

impl DynamicHeuristicParams {

    pub fn default() -> DynamicHeuristicParams {
        DynamicHeuristicParams {
            active_worker_height_to_neighbor_height_to_valuation: [
                [0.0, 0.5, 0.0, 0.0, 0.0], //Start height 0
                [0.0, 0.5, 3.0, 0.0, 0.0], //Start height 1
                [0.0, 0.5, 4.0, f32::MAX, 0.0], //Start height 2
            ],
            active_worker_height_to_valuation: [
                0.0, //height 0
                1.0, //height 1
                4.0, //height 2
            ],
            inactive_worker_height_to_neighbor_height_to_valuation: [
                [0.0, 0.5, 0.3, 0.3, 0.0], //Start height 0
                [0.0, 0.5, 1.0, 0.5, 0.0], //Start height 1
                [0.0, 0.5, 3.0, 4.0, 0.0], //Start height 2
            ],
            inactive_worker_height_to_valuation: [
                0.0, //height 0
                1.0, //height 1
                2.0, //height 2
            ],
            center_distance_valuations: [
                1.0, // Distance 0
                0.5, // Distance 1
                0.0, // Distance sqrt(2)
                0.0, // Distance 2
                0.0, // Distance sqrt(5)
                -0.5, // Distance sqrt(8)
            ],
        }
    }

    pub fn previous_best() -> Self {
        DynamicHeuristicParams { active_worker_height_to_neighbor_height_to_valuation: [[0.4268508, 0.5233252, 1.3759856, -0.14546615, -1.3865342], [0.66122514, 1.2963992, 2.9167352, 0.1304981, -1.7333779], [0.11412573, 1.567106, 3.9054043, 3.4028235e38, -0.71607655]], active_worker_height_to_valuation: [-0.4626242, 1.5078536, 2.187613], inactive_worker_height_to_neighbor_height_to_valuation: [[-0.12419908, 0.5478782, 0.6084795, -0.39584374, -0.39670452], [-0.32223988, -0.02097416, 0.58155036, 0.9824396, 0.40566862], [0.084490955, 0.5915398, 1.835392, 8.850296, 0.2571855]], inactive_worker_height_to_valuation: [-0.066720545, 1.6709696, 2.0311766], center_distance_valuations: [2.244304, 0.6013258, -0.046252787, -0.6884498, -0.24862441, 0.37250865] }
    }

    pub fn get_param_value_count(&self) -> usize {
        return 42;
    }

    pub fn get_value_at_index(&self, index: usize) -> f32 {
        if index < 15 {
            return self.active_worker_height_to_neighbor_height_to_valuation[index / 5][index % 5];
        } else if index < 18 {
            return self.active_worker_height_to_valuation[index - 15];
        } else if index < 33 {
            return self.inactive_worker_height_to_neighbor_height_to_valuation[(index - 18) / 5][(index - 18) % 5];
        } else if index < 36 {
            return self.inactive_worker_height_to_valuation[index - 33];
        } else if index < 42 {
            return self.center_distance_valuations[index - 36];
        } else {
            panic!("Index out of bounds");
        }
    }

    pub fn set_value_at_index(&mut self, index: usize, value: f32) {
        if index < 15 {
            self.active_worker_height_to_neighbor_height_to_valuation[index / 5][index % 5] = value;
        } else if index < 18 {
            self.active_worker_height_to_valuation[index - 15] = value;
        } else if index < 33 {
            self.inactive_worker_height_to_neighbor_height_to_valuation[(index - 18) / 5][(index - 18) % 5] = value;
        } else if index < 36 {
            self.inactive_worker_height_to_valuation[index - 33] = value;
        } else if index < 42 {
            self.center_distance_valuations[index - 36] = value;
        } else {
            panic!("Index out of bounds");
        }
    }

}


const TILE_CENTER_INDEX: [usize; 25] = [
    5, 4, 3, 4, 5,
    4, 2, 1, 2, 4,
    3, 1, 0, 1, 3,
    4, 2, 1, 2, 4,
    5, 4, 3, 4, 5,
];

pub fn dynamic_heuristic<GS: GameState + SantoriniEval<SantoriniState=SantoriniState5x5>>(state: &GS, params: &DynamicHeuristicParams) -> f32 {
    let state = state.get_santorini_state();

    if state.worker_b_tiles[0] == 16 {
        // Setup stage is not supported
        return 0.0;
    }
    debug_assert!(state.tile_heights[state.worker_a_tiles[0] as usize] != 3 && state.tile_heights[state.worker_a_tiles[1] as usize] != 3);
    debug_assert!(state.tile_heights[state.worker_b_tiles[0] as usize] != 3 && state.tile_heights[state.worker_b_tiles[1] as usize] != 3);


    let mut valuation = 0.0;


    for a_worker_tile in &state.worker_a_tiles {
        let worker_tile = *a_worker_tile as usize;
        let worker_height = state.tile_heights[worker_tile] as usize;

        let center_index = TILE_CENTER_INDEX[worker_tile];
        valuation += params.center_distance_valuations[center_index];

        if state.player_a_turn {
            for neighbor_tile in &TILE_TO_NEIGHBORS[worker_tile] {
                if *neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[*neighbor_tile] as usize;
                valuation += params.active_worker_height_to_neighbor_height_to_valuation[worker_height][neighbor_tile_height];
            }
            valuation += params.active_worker_height_to_valuation[worker_height];
        } else {
            for neighbor_tile in &TILE_TO_NEIGHBORS[worker_tile] {
                if *neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[*neighbor_tile] as usize;
                valuation += params.inactive_worker_height_to_neighbor_height_to_valuation[worker_height][neighbor_tile_height];
            }
            valuation += params.inactive_worker_height_to_valuation[worker_height];
        }
    }

    for b_worker_tile in &state.worker_b_tiles {
        let worker_tile = *b_worker_tile as usize;
        let worker_height = state.tile_heights[worker_tile] as usize;

        let center_index = TILE_CENTER_INDEX[worker_tile];
        valuation -= params.center_distance_valuations[center_index];

        if state.player_a_turn {
            for neighbor_tile in &TILE_TO_NEIGHBORS[worker_tile] {
                if *neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[*neighbor_tile] as usize;
                valuation -= params.inactive_worker_height_to_neighbor_height_to_valuation[worker_height][neighbor_tile_height];
            }
            valuation -= params.inactive_worker_height_to_valuation[worker_height];
        } else {
            for neighbor_tile in &TILE_TO_NEIGHBORS[worker_tile] {
                if *neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[*neighbor_tile] as usize;
                valuation -= params.active_worker_height_to_neighbor_height_to_valuation[worker_height][neighbor_tile_height];
            }
            valuation -= params.active_worker_height_to_valuation[worker_height];
        }
    }

    return valuation;
}