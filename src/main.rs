// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]


use std::time::{Duration, Instant};
use num_format::{Locale, ToFormattedString};
use santorini_minimax::game_state::{ContinuousBlockId, GameState, SimplifiedState, StaticEvaluation};
use santorini_minimax::game_state::binary_3bit_game_state::Binary3BitGameState;
use santorini_minimax::generic_game_state::generic_4x4_game_state::Generic4x4GameState;
use santorini_minimax::minimax::minimax_cache::MinimaxCache;
use santorini_minimax::minimax::{minimax, minimax_with_moves, readable_minmax_value};
use santorini_minimax::play_game::play_game;
use santorini_minimax::precompute_state_winner::presolve_state_winner;
use santorini_minimax::strategy::console_input_strategy::ConsoleInputStrategy;
use santorini_minimax::strategy::random_strategy::RandomStrategy;

fn measure_minimax_and_log_moves<GS: GameState + StaticEvaluation + SimplifiedState>(game_state: &GS, depth: usize) {
    let mut minimax_cache = MinimaxCache::new();
    let start = Instant::now();
    let result = minimax_with_moves(game_state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut minimax_cache);
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
    type GS = Binary3BitGameState;

    let mut console_input_strategy = ConsoleInputStrategy::<Generic4x4GameState>::new();
    let mut random_strategy = RandomStrategy::<Generic4x4GameState>::new();

    let generic_game_state = Generic4x4GameState::new(0, 10, [0,0,0,4,0,0,0,4,0,0,0,4,4,4,4,4]).unwrap();
    let game_state = GS::from_generic_game_state(&generic_game_state);

    let result = play_game(&mut console_input_strategy, &mut random_strategy, game_state);

    if result {
        println!("Player 1 wins!");
    } else {
        println!("Player 2 wins!");
    }

    /*
    let num_threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    //let data_folder_path = "/mnt/data/santorini_winner_data";
    let data_folder_path = "winner_data";

    for block_count in (53..=60).rev() {
        println!("Starting presolve for block {}...", block_count);
        presolve_state_winner::<GS>(block_count, num_threads - 4, data_folder_path).await.unwrap();
    }
    */


    /*
    let mut states = Vec::new();
    let block_num = 50;
    println!("Checking count {}", GS::get_continuous_block_id_count(block_num));
    for i in 0..GS::get_continuous_block_id_count(block_num) {
        if i % 1000000 == 0 {
            println!("Progress: {}", i);
        }
        let state = GS::from_continuous_block_id(block_num, i);
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


    //let generic_game_state = Generic4x4GameState::new(0, 10, [0,0,0,4,0,0,0,4,0,0,0,4,4,4,4,4]).unwrap();
    let generic_game_state = Generic4x4GameState::new(5, 9, [1, 1, 1, 0, 0, 0, 1, 4, 4, 0, 1, 1, 0, 1, 2, 4]).unwrap();
    let game_state = GS::from_generic_game_state(&generic_game_state);


    //measure_minimax_and_log_moves(&game_state, 10);


    let start = Instant::now();
    let mut minimax_cache = MinimaxCache::new();
    let value = minimax(&game_state, 50, f32::MIN, f32::MAX, &mut minimax_cache);
    //let value = increasing_depth_minimax(&game_state, 35, Duration::from_secs(600), &mut minimax_cache);
    let duration = start.elapsed();

    println!("Minimax value: {}, took: {:?}", readable_minmax_value(value), duration);
    println!("Evaluated states: {}, pruned states: {}", minimax_cache.evaluated_states.to_formatted_string(&Locale::en), minimax_cache.pruned_states.to_formatted_string(&Locale::en));
    */
}
