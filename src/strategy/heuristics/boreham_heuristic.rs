use crate::game_state::{GameState, SantoriniEval, SantoriniState5x5};
use crate::strategy::heuristics::{NO_NEIGHBOR, TILE_TO_NEIGHBORS};

fn get_height_diff_score(height: u8) -> isize {
    match height {
        0 => 0,
        1 => 40,
        2 => 60,
        3 => 1000,
        _ => panic!("Invalid worker height"),
    }
}

fn get_vertical_mobility_score(worker_height: u8, tile_height: u8) -> isize {
    let height_diff = tile_height as isize - worker_height as isize;
    return match height_diff {
        1 => 5,
        2 => -10,
        3 => -15,
        4 => -20,
        _ => 0,
    };
}

pub fn boreham_heuristic<GS: GameState + SantoriniEval<SantoriniState = SantoriniState5x5>>(state: &GS) -> f32 {
    let state = state.get_santorini_state();

    if state.worker_b_tiles[0] == 16 {
        // Setup stage is not supported
        return 0.0;
    }

    let mut height_diff_score = 0;
    let mut vertical_mobility_score = 0;
    let mut center_square_score = 0;
    let mut lvl2_threat_score = 0;

    for a_worker in state.worker_a_tiles {
        let worker_tile_height = state.tile_heights[a_worker as usize];
        height_diff_score += get_height_diff_score(worker_tile_height);

        for neighbor_tile in TILE_TO_NEIGHBORS[a_worker as usize] {
            if neighbor_tile == NO_NEIGHBOR {
                break;
            }
            let neighbor_tile_height = state.tile_heights[neighbor_tile];
            vertical_mobility_score += get_vertical_mobility_score(worker_tile_height, neighbor_tile_height);
        }

        if a_worker == 12 {
            center_square_score += 15;
        }

        if worker_tile_height == 2 {
            let mut adjacent_lvl3_count = 0;
            for neighbor_tile in TILE_TO_NEIGHBORS[a_worker as usize] {
                if neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[neighbor_tile];
                if neighbor_tile_height == 3 {
                    adjacent_lvl3_count += 1;
                }
            }
            if adjacent_lvl3_count >= 2 {
                lvl2_threat_score += 500;
            }
        }
    }

    for b_worker in state.worker_b_tiles {
        let worker_tile_height = state.tile_heights[b_worker as usize];
        height_diff_score -= get_height_diff_score(worker_tile_height);

        for neighbor_tile in TILE_TO_NEIGHBORS[b_worker as usize] {
            if neighbor_tile == NO_NEIGHBOR {
                break;
            }
            let neighbor_tile_height = state.tile_heights[neighbor_tile];
            vertical_mobility_score -= get_vertical_mobility_score(worker_tile_height, neighbor_tile_height);
        }

        if b_worker == 12 {
            center_square_score -= 15;
        }

        if worker_tile_height == 2 {
            let mut adjacent_lvl3_count = 0;
            for neighbor_tile in TILE_TO_NEIGHBORS[b_worker as usize] {
                if neighbor_tile == NO_NEIGHBOR {
                    break;
                }
                let neighbor_tile_height = state.tile_heights[neighbor_tile];
                if neighbor_tile_height == 3 {
                    adjacent_lvl3_count += 1;
                }
            }
            if adjacent_lvl3_count >= 2 {
                lvl2_threat_score -= 500;
            }
        }
    }

    return (height_diff_score + vertical_mobility_score + center_square_score + lvl2_threat_score) as f32;
}