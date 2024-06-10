// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]

mod game_state;
mod minimax;

use std::time::Instant;
use num_format::{Locale, ToFormattedString};
use crate::game_state::GameState;
use crate::game_state::generic_game_state::GenericGameState;
use crate::minimax::minimax;
use crate::minimax::minimax_cache::MinimaxCache;

fn main() {
    //let generic_state = GenericGameState::new(1, 15, [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]).unwrap();
    // let generic_state = GenericGameState::new(5, 10, [2,2,1,2,2,2,2,1,1,0,1,4,2,0,0,0]).unwrap(); TODO check
    let generic_state = GenericGameState::new(5, 10, [2,2,1,2,2,2,2,1,1,0,1,4,2,0,0,0]).unwrap();
    println!("{}", generic_state);
    let game_state = GameState::from_generic_game_state(&generic_state);

    let mut minimax_cache = MinimaxCache::new();
    let start = Instant::now();
    let val = minimax(&game_state, 7, f32::NEG_INFINITY, f32::INFINITY, &mut minimax_cache);
    let duration = start.elapsed();
    println!("Minimax value: {}, took: {:?}", val, duration);

    println!("Evaluated states: {}, pruned states: {}", minimax_cache.evaluated_states.to_formatted_string(&Locale::en), minimax_cache.pruned_states.to_formatted_string(&Locale::en));
    println!("Pruned percentage: {}", minimax_cache.pruned_states as f32 / (minimax_cache.evaluated_states + minimax_cache.pruned_states) as f32);
}
