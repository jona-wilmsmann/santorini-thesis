// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]


use std::time::{Duration, Instant};
use num_format::{Locale, ToFormattedString};
use santorini_minimax::game_state::{ContinuousBlockId, GameState, SimplifiedState, MinimaxReady};
use santorini_minimax::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use santorini_minimax::game_state::game_state_5x5_binary_128bit::GameState5x5Binary128bit;
use santorini_minimax::game_state::game_state_5x5_struct::GameState5x5Struct;
use santorini_minimax::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use santorini_minimax::generic_game_state::GenericGameState;
use santorini_minimax::minimax::minimax_cache::MinimaxCache;
use santorini_minimax::minimax::{minimax, readable_minmax_value};
use santorini_minimax::play_game::play_game;
use santorini_minimax::precompute_state_winner::presolve_state_winner;
use santorini_minimax::stats::game_states_by_block_count::GameStatesByBlockCount;
use santorini_minimax::stats::StatGenerator;
use santorini_minimax::strategy::console_input_strategy::ConsoleInputStrategy;
use santorini_minimax::strategy::random_strategy::RandomStrategy;
use anyhow::Result;

/*
fn measure_minimax_and_log_moves<GS: GameState + MinimaxReady + SimplifiedState>(game_state: &GS, depth: usize) {
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
 */

fn store_game_state_image<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize>
(game_state: &GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER>, name: &str) -> Result<()> {
    let folder_path = "stats/game_states/";
    std::fs::create_dir_all(folder_path.clone())?;
    let image_path = format!("{}/{}.svg", folder_path, name);
    game_state.draw_image(image_path.as_str())?;
    return Ok(());
}

#[tokio::main]
async fn main() {
    type GS5x5 = GameState5x5Struct;
    type GGS5x5 = <GameState5x5Struct as GameState>::GenericGameState;

    let mut max_children = 0;
    let mut max_children_state = None;
    for worker_a_1 in 0..25 {
        for worker_a_2 in 0..25 {
            for worker_b_1 in 0..25 {
                for worker_b_2 in 0..25 {
                    if worker_a_1 == worker_a_2 || worker_a_1 == worker_b_1 || worker_a_1 == worker_b_2 || worker_a_2 == worker_b_1 || worker_a_2 == worker_b_2 || worker_b_1 == worker_b_2 {
                        continue;
                    }
                    let generic_game_state = GGS5x5::new(
                        [worker_a_1, worker_a_2],
                        [worker_b_1, worker_b_2],
                        [
                            [0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0],
                            [0, 0, 0, 0, 0]
                        ],
                        true,
                    ).unwrap();
                    let game_state = GS5x5::from_generic_game_state(&generic_game_state);
                    let children_number = game_state.get_children_states().len();
                    if children_number > max_children {
                        max_children = children_number;
                        max_children_state = Some(generic_game_state);
                    }
                }
            }
        }
    }
    let max_children_state = max_children_state.unwrap();

    println!("Max children: {}", max_children);
    println!("State: {}", max_children_state);
    store_game_state_image(&max_children_state, "max_children").unwrap();

    /*
    type GS4x4 = GameState4x4Binary3Bit;

    let num_threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let data_folder_path = "/mnt/data/santorini_winner_data_new";
    let data_folder_path_draw = "/mnt/data/santorini_winner_data_draw";
    //let data_folder_path = "winner_data";

    for block_count in (0..=60).rev() {
        println!("Starting draw presolve for block {}...", block_count);
        presolve_state_winner::<GS4x4, 2>(block_count, num_threads - 4, data_folder_path_draw).await.unwrap();
    }

    for block_count in (0..=60).rev() {
        println!("Starting presolve for block {}...", block_count);
        presolve_state_winner::<GS4x4, 1>(block_count, num_threads - 4, data_folder_path).await.unwrap();
    }

     */

    /*
    type GS4x4 = GameState4x4Binary3Bit;
    type GGS4x4 = <GameState4x4Binary3Bit as GameState>::GenericGameState;

    let generic_game_state_4x4 = GGS4x4::new(
        [0],
        [15],
        [
            [0, 1, 2, 2],
            [1, 1, 3, 0],
            [0, 2, 4, 2],
            [0, 1, 4, 0],
        ],
        true,
    ).unwrap();

    generic_game_state_4x4.draw_image("test.svg").unwrap();

    type GS5x5 = GameState5x5Struct;
    type GGS5x5 = <GameState5x5Struct as GameState>::GenericGameState;

    let generic_game_state_5x5 = GGS5x5::generate_random_state_with_blocks(15);

    generic_game_state_5x5.draw_image("test2.svg").unwrap();

     */

    /*
    type GS4x4 = GameState4x4Binary3Bit;
    type GGS4x4 = <GameState4x4Binary3Bit as GameState>::GenericGameState;

    let generic_game_state_4x4 = GGS4x4::new(
        [0],
        [15],
        [
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
        ],
        true,
    ).unwrap();
    let game_state_4x4 = GS4x4::from_generic_game_state(&generic_game_state_4x4);

    type GS5x5 = GameState5x5Struct;
    type GGS5x5 = <GameState5x5Struct as GameState>::GenericGameState;

    let generic_game_state_5x5 = GGS5x5::new(
        [0, 1],
        [23, 24],
        [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ],
        true,
    ).unwrap();
    let game_state_5x5 = GS5x5::from_generic_game_state(&generic_game_state_5x5);


    let depth = 4;


    let start = Instant::now();
    let mut minimax_cache = MinimaxCache::new();
    let value = minimax(&game_state_4x4, depth, f32::MIN, f32::MAX, &mut minimax_cache);

    let duration = start.elapsed();

    println!("4x4");
    println!("Minimax value: {}, took: {:?}", readable_minmax_value(value), duration);
    println!("Evaluated states: {}, pruned states: {}", minimax_cache.evaluated_states.to_formatted_string(&Locale::en), minimax_cache.pruned_states.to_formatted_string(&Locale::en));



    let start = Instant::now();
    let mut minimax_cache = MinimaxCache::new();
    let value = minimax(&game_state_5x5, depth, f32::MIN, f32::MAX, &mut minimax_cache);

    let duration = start.elapsed();

    println!("5x5");
    println!("Minimax value: {}, took: {:?}", readable_minmax_value(value), duration);
    println!("Evaluated states: {}, pruned states: {}", minimax_cache.evaluated_states.to_formatted_string(&Locale::en), minimax_cache.pruned_states.to_formatted_string(&Locale::en));

     */

    /*
    type GGS = GenericSantoriniGameState<5, 5, 2>;
    type GS = GameState5x5Binary128bit;

    let generic_game_state = GGS::new(
        [0, GGS::WORKER_NOT_PLACED],
        [23, GGS::WORKER_NOT_PLACED],
        [
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ],
        true,
    ).unwrap();

    println!("{}", generic_game_state);

    let game_state = GS::from_generic_game_state(&generic_game_state);

    let children_states = game_state.get_children_states();

    for (i, state) in children_states.iter().enumerate() {
        println!("Child state {}:\n{}", i, state);
    }

     */

    /*
    type GS = GameState4x4Binary3Bit;

    let mut console_input_strategy = ConsoleInputStrategy::<GenericSantoriniGameState<4, 4, 1>>::new();
    let mut random_strategy = RandomStrategy::<GenericSantoriniGameState<4, 4, 1>>::new();

    let generic_game_state = GenericSantoriniGameState::<4, 4, 1>::new([0], [10], [[0, 0, 0, 4], [0, 0, 0, 4], [0, 0, 0, 4], [4, 4, 4, 4]], true).unwrap();
    let game_state = GS::from_generic_game_state(&generic_game_state);

    let result = play_game(&mut console_input_strategy, &mut random_strategy, game_state);

    if result {
        println!("Player 1 wins!");
    } else {
        println!("Player 2 wins!");
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
    */
}
