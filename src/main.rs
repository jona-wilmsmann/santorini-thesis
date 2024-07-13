// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]

mod game_state;
mod minimax;
mod measurements;
mod precompute_state_winner;

use std::time::{Duration, Instant};
use num_format::{Locale, ToFormattedString};
use crate::game_state::GameState;
use crate::game_state::generic_game_state::GenericGameState;
use crate::game_state::utils::random_state_generation::generate_random_state;
use crate::measurements::create_csv_report::create_csv_report;
use crate::minimax::{readable_minmax_value, minimax_with_moves, minimax, increasing_depth_minimax};
use crate::minimax::minimax_cache::MinimaxCache;
use crate::precompute_state_winner::{presolve_state_winner};

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

#[tokio::main]
async fn main() {
    let num_threads = std::thread::available_parallelism().map_or(1, |n| n.get());

    for block_count in (0..=60).rev() {
        println!("Starting presolve for block {}...", block_count);
        presolve_state_winner(block_count, num_threads - 4).await.unwrap();
    }


    return;
    let mut states = Vec::new();
    let block_num = 50;
    println!("Checking count {}", GameState::get_continuous_block_id_count(block_num));
    for i in 0..GameState::get_continuous_block_id_count(block_num) {
        if i % 1000000 == 0 {
            println!("Progress: {}", i);
        }
        let state = GameState::from_continuous_block_id(block_num, i);
        let continuous_block_id = state.get_continuous_block_id();
        if continuous_block_id != i {
            println!("Mismatch: {} != {}", continuous_block_id, i);
            break;
        }
        states.push(state.raw_value());
    }

    states.sort();
    states.dedup();

    println!("States: {}", states.len());
    return;


    //create_csv_report(100, 15..=50, 1..=10).await.unwrap();


    //let generic_game_state = GenericGameState::new(0, 10, [0,0,0,4,0,0,0,4,0,0,0,4,4,4,4,4]).unwrap();
    let generic_game_state = GenericGameState::new(5, 9, [1, 1, 1, 0, 0, 0, 1, 4, 4, 0, 1, 1, 0, 1, 2, 4]).unwrap();
    let game_state = GameState::from_generic_game_state(&generic_game_state);


    //measure_minimax_and_log_moves(&game_state, 10);


    let start = Instant::now();
    let mut minimax_cache = MinimaxCache::new();
    let value = minimax(&game_state, 50, f32::MIN, f32::MAX, &mut minimax_cache);
    //let value = increasing_depth_minimax(&game_state, 35, Duration::from_secs(600), &mut minimax_cache);
    let duration = start.elapsed();

    println!("Minimax value: {}, took: {:?}", readable_minmax_value(value), duration);
    println!("Evaluated states: {}, pruned states: {}", minimax_cache.evaluated_states.to_formatted_string(&Locale::en), minimax_cache.pruned_states.to_formatted_string(&Locale::en));
}
