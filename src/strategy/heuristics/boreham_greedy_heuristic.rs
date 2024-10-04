use crate::game_state::{GameState, SantoriniEval, SantoriniState5x5};

pub fn boreham_greedy_heuristic<GS: GameState + SantoriniEval<SantoriniState = SantoriniState5x5>>(state: &GS) -> f32 {
    let state = state.get_santorini_state();

    if state.worker_b_tiles[0] == 16 {
        // Setup stage is not supported
        return 0.0;
    }

    let mut score = 0;

    for a_worker in state.worker_a_tiles {
        let worker_tile_height = state.tile_heights[a_worker as usize];
        score += worker_tile_height as isize;
    }

    for b_worker in state.worker_b_tiles {
        let worker_tile_height = state.tile_heights[b_worker as usize];
        score -= worker_tile_height as isize;
    }

    return score as f32;
}