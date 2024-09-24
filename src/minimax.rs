pub mod minimax_cache;

use crate::game_state::{GameState, SimplifiedState, MinimaxReady};
use crate::minimax::minimax_cache::{Bounds, MinimaxCache};


fn simple_minimax_internal<GS: GameState>(game_state: &GS, maximizing_player: bool, depth: usize, reused_children_vec: &mut Vec<GS>, evaluated_states: &mut usize) -> f32 {
    *evaluated_states += 1;

    if game_state.has_player_a_won() {
        return f32::MAX;
    } else if game_state.has_player_b_won() {
        return f32::MIN;
    }

    if depth == 0 {
        return 0.0;
    }

    game_state.get_children_states_reuse_vec(reused_children_vec);

    let mut reusable_vec_for_children = Vec::with_capacity(32);
    if maximizing_player {
        let mut max_evaluation = f32::NEG_INFINITY;
        for child in reused_children_vec {
            let evaluation = simple_minimax_internal(child, false, depth - 1, &mut reusable_vec_for_children, evaluated_states);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
            }
        }
        return max_evaluation;
    } else {
        let mut min_evaluation = f32::INFINITY;
        for child in reused_children_vec {
            let evaluation = simple_minimax_internal(child, true, depth - 1, &mut reusable_vec_for_children, evaluated_states);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
            }
        }
        return min_evaluation;
    }
}

pub fn simple_minimax<GS: GameState>(game_state: &GS, depth: usize) -> (f32, usize) {
    let mut evaluated_states = 0;
    let mut reused_children_vec = Vec::with_capacity(32);
    let result = simple_minimax_internal(game_state, game_state.is_player_a_turn(), depth, &mut reused_children_vec, &mut evaluated_states);
    return (result, evaluated_states);
}


fn alpha_beta_minimax_internal<GS: GameState>(game_state: &GS, maximizing_player: bool, depth: usize, mut alpha: f32, mut beta: f32, reused_children_vec: &mut Vec<GS>, evaluated_states: &mut usize) -> f32 {
    *evaluated_states += 1;

    if game_state.has_player_a_won() {
        return f32::MAX;
    } else if game_state.has_player_b_won() {
        return f32::MIN;
    }

    if depth == 0 {
        return 0.0;
    }

    game_state.get_children_states_reuse_vec(reused_children_vec);

    let mut reusable_vec_for_children = Vec::with_capacity(32);
    if maximizing_player {
        let mut max_evaluation = f32::NEG_INFINITY;
        for child in reused_children_vec {
            let evaluation = alpha_beta_minimax_internal(child, false, depth - 1, alpha, beta, &mut reusable_vec_for_children, evaluated_states);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
                if max_evaluation >= beta {
                    break;
                }
                if max_evaluation > alpha {
                    alpha = max_evaluation;
                }
            }
        }
        return max_evaluation;
    } else {
        let mut min_evaluation = f32::INFINITY;
        for child in reused_children_vec {
            let evaluation = alpha_beta_minimax_internal(child, true, depth - 1, alpha, beta, &mut reusable_vec_for_children, evaluated_states);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
                if min_evaluation <= alpha {
                    break;
                }
                if min_evaluation < beta {
                    beta = min_evaluation;
                }
            }
        }
        return min_evaluation;
    }
}

pub fn alpha_beta_minimax<GS: GameState>(game_state: &GS, depth: usize) -> (f32, usize) {
    let mut evaluated_states = 0;
    let mut reused_children_vec = Vec::with_capacity(32);
    let result = alpha_beta_minimax_internal(game_state, game_state.is_player_a_turn(), depth, f32::NEG_INFINITY, f32::INFINITY, &mut reused_children_vec, &mut evaluated_states);
    return (result, evaluated_states);
}

pub fn minimax<GS: GameState + MinimaxReady>(game_state: &GS, depth: usize, mut alpha: f32, mut beta: f32, cache: &mut MinimaxCache<GS>) -> f32 {
    cache.evaluated_states += 1;

    if depth == 0 {
        return game_state.get_static_evaluation();
    }

    if game_state.has_player_a_won() {
        return f32::MAX;
    } else if game_state.has_player_b_won() {
        return f32::MIN;
    }


    let mut children_states = game_state.get_children_states();
    // TODO: This speeds things up for some states, but makes things slower for others. Think of ways to detect when to use it
    //children_states = children_states.iter().map(|child| child.get_simplified_state()).collect();
    GS::sort_children_states(&mut children_states, depth, cache);

    if children_states.len() == 0 {
        return if game_state.is_player_a_turn() {
            f32::MIN
        } else {
            f32::MAX
        }
    }

    /*
    // TODO: Double check if this all makes sense and if there are any other possible cases to cover
    if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
        if cached_value.alpha <= alpha && cached_value.beta >= beta {
            return cached_value.value;
        }
        if cached_value.value >= beta {
            return cached_value.value;
        }

        if cached_value.beta <= beta {
            if cached_value.value > alpha {
                alpha = cached_value.value;
                max_evaluation = alpha;
            }
        }
    }

     */


    let mut evaluated_children = 0;
    if game_state.is_player_a_turn() {
        // Player A is maximizing
        let mut max_evaluation = f32::NEG_INFINITY;

        if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
            if cached_value.alpha <= alpha && cached_value.beta >= beta {
                return cached_value.value;
            }
            if cached_value.value >= beta {
                return cached_value.value;
            }

            if cached_value.beta <= beta {
                if cached_value.value > alpha {
                    alpha = cached_value.value;
                    max_evaluation = alpha;
                }
            }
        }

        for child in &children_states {
            evaluated_children += 1;
            let evaluation = minimax(child, depth - 1, alpha, beta, cache);
            if evaluation > max_evaluation {
                max_evaluation = evaluation;
            }
            if evaluation > alpha {
                alpha = evaluation;
            }
            if beta <= alpha {
                break;
            }
        }

        cache.pruned_states += children_states.len() - evaluated_children;

        cache.insert_valuation_bounds(depth, game_state.clone(), Bounds { value: max_evaluation, alpha, beta });

        return max_evaluation;

    } else {
        // Player B is minimizing
        let mut min_evaluation = f32::INFINITY;

        if let Some(cached_value) = cache.get_valuation_bounds(depth, game_state) {
            if cached_value.alpha <= alpha && cached_value.beta >= beta {
                return cached_value.value;
            }
            if cached_value.value <= alpha {
                return cached_value.value;
            }

            if cached_value.alpha >= alpha {
                if cached_value.value < beta {
                    beta = cached_value.value;
                    min_evaluation = beta;
                }
            }
        }

        for child in &children_states {
            evaluated_children += 1;
            let evaluation = minimax(child, depth - 1, alpha, beta, cache);
            if evaluation < min_evaluation {
                min_evaluation = evaluation;
            }
            if evaluation < beta {
                beta = evaluation;
            }
            if beta <= alpha {
                break;
            }
        }

        cache.pruned_states += children_states.len() - evaluated_children;

        cache.insert_valuation_bounds(depth, game_state.clone(), Bounds { value: min_evaluation, alpha, beta });

        return min_evaluation;
    }
}