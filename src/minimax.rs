pub mod minimax_cache;

use std::time::Duration;
use crate::game_state::GameState;
use crate::minimax::minimax_cache::MinimaxCache;

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
    }
}

// Allows for flexibly adding caching if needed
fn get_static_evaluation(game_state: &GameState, cache: &mut MinimaxCache) -> f32 {
    //return game_state.static_evaluation();

    if let Some(cached_value) = cache.static_valuations.get(game_state) {
        return *cached_value;
    }
    let static_evaluation = game_state.static_evaluation();
    cache.static_valuations.insert(game_state.clone(), static_evaluation);
    return static_evaluation;
}

fn sort_children_states(parent_state: &GameState, children_states: &mut Vec<GameState>, depth: usize, cache: &mut MinimaxCache) {
    if depth > 2 {
        // Create a vector of tuples with the static evaluation and the GameState
        let mut children_evaluations: Vec<(GameState, f32)> = children_states.iter().map(|state| (state.clone(), get_static_evaluation(state, cache))).collect();
        // Sort the vector by the static evaluation
        children_evaluations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        // Replace the children_states vector with the sorted vector
        *children_states = children_evaluations.iter().map(|(state, _)| state.clone()).collect();
    }

    if let Some(best_child) = cache.best_child.get(parent_state) {
        if let Some(index) = children_states.iter().position(|state| state == best_child) {
            if index != 0 {
                let best_child = children_states.remove(index);
                children_states.insert(0, best_child);
            }
        }
    }
}

pub fn minimax(game_state: &GameState, depth: usize, mut alpha: f32, beta: f32, cache: &mut MinimaxCache) -> f32 {
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
    // to symmetric_transpose
    //children_states = children_states.iter().map(|child| child.get_symmetric_simplified_state()).collect();
    sort_children_states(game_state, &mut children_states, depth, cache);

    if children_states.len() == 0 {
        return f32::MIN
    }

    let mut max_evaluation = f32::NEG_INFINITY;
    let mut best_child_state = None;
    let mut evaluated_children = 0;
    for child in &children_states {
        evaluated_children += 1;
        let flipped_state = child.get_flipped_state();
        let evaluation = -minimax(&flipped_state, depth - 1, -beta, -alpha, cache);
        if evaluation > max_evaluation {
            max_evaluation = evaluation;
            best_child_state = Some(child.clone());
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
        if alpha >= beta {
            break;
        }
    }
    if let Some(best_child) = best_child_state {
        cache.best_child.insert(game_state.clone(), best_child);
    }

    cache.pruned_states += children_states.len() - evaluated_children;

    return max_evaluation;
}


pub fn increasing_depth_minimax(game_state: &GameState, max_depth: usize, cutoff_time: Duration, cache: &mut MinimaxCache) -> f32 {
    let start = std::time::Instant::now();
    let mut value = 0.0;
    let mut reached_depth = 0;

    let mut last_evaluated_states = 0;
    let mut last_pruned_states = 0;

    for depth in 1..=max_depth {
        let duration = start.elapsed();
        if duration > cutoff_time {
            break;
        }
        value = minimax(game_state, depth, f32::NEG_INFINITY, f32::INFINITY, cache);
        reached_depth = depth;
        println!("Depth: {}, value: {}, Evaluated states: {}, Pruned states: {}, Time elapsed: {:?}", depth, readable_minmax_value(value), cache.evaluated_states - last_evaluated_states, cache.pruned_states - last_pruned_states, start.elapsed());
        last_evaluated_states = cache.evaluated_states;
        last_pruned_states = cache.pruned_states;
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

pub fn minimax_with_moves(game_state: &GameState, depth: usize, mut alpha: f32, beta: f32, cache: &mut MinimaxCache) -> (f32, Vec<GameState>) {
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
    sort_children_states(game_state, &mut children_states, depth, cache);

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

    return (move_f32_closer_to_zero(max_evaluation), game_state_path)
}