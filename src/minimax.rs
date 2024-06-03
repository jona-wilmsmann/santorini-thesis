pub mod minimax_cache;

use crate::game_state::GameState;
use crate::minimax::minimax_cache::MinimaxCache;

pub fn minimax(game_state: &GameState, depth: usize, mut alpha: f32, beta: f32, cache: &mut MinimaxCache) -> f32 {
    cache.evaluated_states += 1;

    if depth == 0 {
        return game_state.static_evaluation();
    }

    /*
    if depth == 0 {
        if let Some(cached_value) = cache.static_valuations.get(game_state) {
            return *cached_value;
        }
        let static_evaluation = game_state.static_evaluation();
        cache.static_valuations.insert(game_state.clone(), static_evaluation);
        return static_evaluation;
    }
     */


    let mut children = game_state.get_possible_next_states();
    // to symmetric_transpose
    //children = children.iter().map(|child| child.symmetric_transpose()).collect();

    if depth > 4 {
        children.sort_by(|a, b| b.static_evaluation().partial_cmp(&a.static_evaluation()).unwrap());
    }

    if children.len() == 0 {
        return if game_state.has_player_a_won() {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        }
    }

    let mut max_evaluation = f32::NEG_INFINITY;
    let mut evaluated_children = 0;
    for child in &children {
        evaluated_children += 1;
        let flipped_state = child.get_flipped_state();
        let evaluation = -minimax(&flipped_state, depth - 1, -beta, -alpha, cache);
        if evaluation > max_evaluation {
            max_evaluation = evaluation;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        if alpha >= beta {
            break;
        }
    }
    cache.pruned_states += children.len() - evaluated_children;

    return max_evaluation
}