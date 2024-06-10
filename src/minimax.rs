pub mod minimax_cache;

use crate::game_state::GameState;
use crate::minimax::minimax_cache::MinimaxCache;

pub fn readable_minmax_value(value: f32) -> String {
    // If the value is close to MAX or MIN
    return if value > 1_000_000.0 {
        let difference = f32::MAX.to_bits() - value.to_bits();
        format!("Winning in {}", difference)
    } else if value < -1_000_000.0 {
        let difference = f32::MIN.to_bits() - value.to_bits();
        format!("Losing in {}", difference)
    } else {
        value.to_string()
    }
}

// Allows for flexibly adding caching if needed
fn get_static_evaluation(game_state: &GameState, cache: &mut MinimaxCache) -> f32 {
    return game_state.static_evaluation();

    if let Some(cached_value) = cache.static_valuations.get(game_state) {
        return *cached_value;
    }
    let static_evaluation = game_state.static_evaluation();
    cache.static_valuations.insert(game_state.clone(), static_evaluation);
    return static_evaluation;
}

fn sort_children_states(children_states: &mut Vec<GameState>, depth: usize) {
    if depth > 4 {
        children_states.sort_by(|a, b| b.static_evaluation().partial_cmp(&a.static_evaluation()).unwrap());
    }
}

// This function is used to prioritize moves that reach a good game state faster, or delay a bad game state
fn move_f32_closer_to_zero(value: f32) -> f32 {
    let mut int_value = value.to_bits();
    int_value -= 1;
    return f32::from_bits(int_value);
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
    // children = children.iter().map(|child| child.symmetric_transpose()).collect();
    sort_children_states(&mut children_states, depth);

    if children_states.len() == 0 {
        return f32::MIN
    }

    let mut max_evaluation = f32::NEG_INFINITY;
    let mut evaluated_children = 0;
    for child in &children_states {
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
    cache.pruned_states += children_states.len() - evaluated_children;

    return max_evaluation
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
    sort_children_states(&mut children_states, depth);

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