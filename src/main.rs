// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]

mod game_state;
mod minimax;

use std::time::Instant;
use num_format::{Locale, ToFormattedString};
use crate::game_state::GameState;
use crate::game_state::generic_game_state::GenericGameState;
use crate::minimax::{readable_minmax_value, minimax, minimax_with_moves};
use crate::minimax::minimax_cache::MinimaxCache;

fn measure_minimax_and_log_moves(game_state: &GameState, depth: usize) {
    let mut minimax_cache = MinimaxCache::new();
    let start = Instant::now();
    let result = minimax_with_moves(&game_state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut minimax_cache);
    let duration = start.elapsed();
    for (i, state) in result.1.iter().enumerate() {
        let corrected_state = if i % 2 == 0 {
            state
        } else {
            &state.get_flipped_state()
        };
        let player = if i % 2 == 0 { "A" } else { "B" };
        println!("State {} (Player {} to move):\n{}", i, player, corrected_state);
    }

    println!("Minimax value: {}, took: {:?}", readable_minmax_value(result.0), duration);
    println!("Evaluated states: {}, pruned states: {}", minimax_cache.evaluated_states.to_formatted_string(&Locale::en), minimax_cache.pruned_states.to_formatted_string(&Locale::en));
}

fn main() {
    //let generic_state = GenericGameState::new(1, 15, [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]).unwrap();
    let generic_state = GenericGameState::new(5, 10, [2, 2, 1, 2, 2, 2, 2, 1, 1, 0, 1, 4, 2, 0, 0, 0]).unwrap();
    let game_state = GameState::from_generic_game_state(&generic_state);

    measure_minimax_and_log_moves(&game_state, 5);
}
