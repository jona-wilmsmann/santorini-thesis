pub mod minimax_cache;

use std::time::Duration;
use num_format::{Locale, ToFormattedString};
use crate::game_state::{GameState, SimplifiedState, MinimaxReady};
use crate::minimax::minimax_cache::{Bounds, MinimaxCache};

pub fn readable_minmax_value(value: f32) -> String {
    // If the value is close to MAX or MIN
    return if value > 1_000_000.0 {
        let difference = f32::MAX.to_bits() - value.to_bits();
        format!("#+{}", difference)
    } else if value < -1_000_000.0 {
        let difference = f32::MIN.to_bits() - value.to_bits();
        format!("#-{}", difference)
    } else {
        value.to_string()
    };
}

// Allows for flexibly adding caching if needed
fn get_static_evaluation<GS: GameState + MinimaxReady>(game_state: &GS, _cache: &mut MinimaxCache<GS>) -> f32 {
    return game_state.get_static_evaluation();

    /*
    if let Some(cached_value) = cache.valuations[0].get(game_state) {
        return *cached_value;
    }
    let static_evaluation = game_state.static_evaluation();
    cache.valuations[0].insert(game_state.clone(), static_evaluation);
    return static_evaluation;
    */
}

pub fn minimax<GS: GameState + MinimaxReady>(game_state: &GS, depth: usize, mut alpha: f32, mut beta: f32, cache: &mut MinimaxCache<GS>) -> f32 {
    cache.evaluated_states += 1;

    if depth == 0 {
        return get_static_evaluation(game_state, cache);
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


pub fn increasing_depth_minimax<GS: GameState + MinimaxReady + SimplifiedState>(game_state: &GS, max_depth: usize, cutoff_time: Duration, cache: &mut MinimaxCache<GS>) -> f32 {
    let start = std::time::Instant::now();
    let mut value = 0.0;
    let mut reached_depth = 0;

    let mut last_evaluated_states = 0;
    let mut last_pruned_states = 0;
    let mut last_time = start;

    for depth in 1..=max_depth {
        let duration = start.elapsed();
        if duration > cutoff_time {
            break;
        }
        value = minimax(game_state, depth, f32::MIN, f32::MAX, cache);
        reached_depth = depth;
        println!("Depth: {}, value: {}, Evaluated states: {}, Pruned states: {}, Took: {:?}", depth, readable_minmax_value(value), (cache.evaluated_states - last_evaluated_states).to_formatted_string(&Locale::en), (cache.pruned_states - last_pruned_states).to_formatted_string(&Locale::en), last_time.elapsed());
        last_evaluated_states = cache.evaluated_states;
        last_pruned_states = cache.pruned_states;
        last_time = std::time::Instant::now();
    }
    return value;
}


// This function is used to prioritize moves that reach a good game state faster, or delay a bad game state
fn move_f32_closer_to_zero(value: f32) -> f32 {
    if value == 0.0 {
        return value;
    }
    let mut int_value = value.to_bits();
    int_value -= 1;
    return f32::from_bits(int_value);
}

/*
TODO: Update this when minimax is stable
pub fn minimax_with_moves<GS: GameState + MinimaxReady + SimplifiedState>(game_state: &GS, depth: usize, mut alpha: f32, beta: f32, cache: &mut MinimaxCache<GS>) -> (f32, Vec<GS>) {
    let mut game_state_path = vec![game_state.clone()];
    cache.evaluated_states += 1;

    if depth == 0 {
        return (get_static_evaluation(game_state, cache), game_state_path);
    }

    if game_state.has_player_a_won() {
        return (f32::MAX, game_state_path);
    } else if game_state.has_player_b_won() {
        return (f32::MIN, game_state_path);
    }

    let mut children_states = game_state.get_children_states();
    sort_children_states(&mut children_states, depth, cache);

    if children_states.len() == 0 {
        return (f32::MIN, game_state_path);
    }

    let mut max_evaluation = f32::NEG_INFINITY;
    let mut best_children_path = vec![];
    let mut evaluated_children = 0;
    for child in &children_states {
        evaluated_children += 1;
        let flipped_state = child.get_flipped_state();
        let result = minimax_with_moves(&flipped_state, depth - 1, -beta, -alpha, cache);
        let evaluation = -result.0;
        if evaluation > max_evaluation {
            best_children_path = result.1;
            max_evaluation = evaluation;
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        if alpha >= beta {
            break;
        }
    }
    cache.pruned_states += children_states.len() - evaluated_children;

    game_state_path.extend(best_children_path);

    return (move_f32_closer_to_zero(max_evaluation), game_state_path);
}

 */