// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]

mod game_state;
mod minimax;
mod measurements;

use std::time::{Duration, Instant};
use num_format::{Locale, ToFormattedString};
use crate::game_state::GameState;
use crate::game_state::generic_game_state::GenericGameState;
use crate::game_state::utils::random_state_generation::generate_random_state;
use crate::measurements::create_csv_report::create_csv_report;
use crate::minimax::{readable_minmax_value, minimax_with_moves, minimax, increasing_depth_minimax};
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

#[tokio::main]
async fn main() {
    /*
    let depth = 3;
    for _ in 0..1000000 {
        let random_state = generate_random_state();
        let game_state = GameState::from_generic_game_state(&random_state);
        let simplified_state = game_state.get_symmetric_simplified_state();

        let original_minimax = minimax(&game_state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());
        let simplified_minimax = minimax(&simplified_state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());

        if original_minimax != simplified_minimax {
            println!("Minimax mismatch!");
            println!("Original state (Minimax: {}):\n{}", original_minimax, game_state);
            println!("Simplified state (Minimax: {}):\n{}", simplified_minimax, simplified_state);

            let (val, moves) = minimax_with_moves(&game_state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());
            for (i, state) in moves.iter().enumerate() {
                let corrected_state = if i % 2 == 0 {
                    state
                } else {
                    &state.get_flipped_state()
                };
                let player = if i % 2 == 0 { "A" } else { "B" };
                println!("State {} (Player {} to move):\n{}", i, player, corrected_state);
            }

            let (val, moves) = minimax_with_moves(&simplified_state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());
            for (i, state) in moves.iter().enumerate() {
                let corrected_state = if i % 2 == 0 {
                    state
                } else {
                    &state.get_flipped_state()
                };
                let player = if i % 2 == 0 { "A" } else { "B" };
                println!("Simplified state {} (Player {} to move):\n{}", i, player, corrected_state);
            }
            break;
        }
    }

     */



    /*
    let generic_game_state = GenericGameState::new(12, 1, [4,2,2,4,3,0,1,0,2,2,2,4,1,4,3,0]).unwrap();
    let game_state = GameState::from_generic_game_state(&generic_game_state);
    let simplified_state = game_state.get_symmetric_simplified_state();

    let core_val = minimax(&game_state, 3, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());
    let simplified_core_val = minimax(&simplified_state, 3, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());


    let (original_val, original_moves) = minimax_with_moves(&game_state, 3, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());
    let (simplified_val, simplified_moves) = minimax_with_moves(&simplified_state, 3, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());

    println!("Original state (Minimax: {}):\n{}", original_val, game_state);
    for (i, state) in original_moves.iter().enumerate() {
        let corrected_state = if i % 2 == 0 {
            state
        } else {
            &state.get_flipped_state()
        };
        let player = if i % 2 == 0 { "A" } else { "B" };
        println!("State {} (Player {} to move):\n{}", i, player, corrected_state);
    }

    println!("Simplified state (Minimax: {}):\n{}", simplified_val, simplified_state);
    for (i, state) in simplified_moves.iter().enumerate() {
        let corrected_state = if i % 2 == 0 {
            state
        } else {
            &state.get_flipped_state()
        };
        let player = if i % 2 == 0 { "A" } else { "B" };
        println!("Simplified state {} (Player {} to move):\n{}", i, player, corrected_state);
    }

    println!("Original minimax: {}, simplified minimax: {}", core_val, simplified_core_val);
    */


    //create_csv_report(100, 15..=50, 1..=10).await.unwrap();


    //let generic_game_state = GenericGameState::new(0, 10, [0,0,0,4,0,0,0,4,0,0,0,4,4,4,4,4]).unwrap();
    let generic_game_state = GenericGameState::new(5, 9, [1,1,1,0,0,0,1,4,4,0,1,1,0,1,2,4]).unwrap();
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
