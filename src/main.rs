// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]

use santorini_minimax::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use santorini_minimax::stats::{StatGenerator};
use anyhow::Result;
use rand::SeedableRng;
use santorini_minimax::game_state::game_state_5x5_binary_composite::GameState5x5BinaryComposite;
use santorini_minimax::game_state::game_state_5x5_struct::GameState5x5Struct;
use santorini_minimax::game_state::{GameState, MinimaxReady};
use santorini_minimax::generic_game_state::GenericGameState;
use santorini_minimax::minimax::{alpha_beta_sorted_minimax};
use santorini_minimax::stats::benchmark_minimax_alpha_beta::BenchmarkMinimaxAlphaBeta;
use santorini_minimax::stats::benchmark_minimax_cached::BenchmarkMinimaxCached;
use santorini_minimax::stats::benchmark_minimax_simple::BenchmarkMinimaxSimple;
use santorini_minimax::stats::benchmark_minimax_sorted::BenchmarkMinimaxSorted;
use santorini_minimax::stats::utils::formatters::ns_formatter;

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
    std::fs::create_dir_all(folder_path)?;
    let image_path = format!("{}/{}.svg", folder_path, name);
    game_state.draw_image(image_path.as_str())?;
    return Ok(());
}

async fn average_branching_factor<GS: GameState + MinimaxReady + 'static>(number_random_states: usize, block_count: usize, depth: usize) -> Result<f32> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let random_states: Vec<GS> = (0..number_random_states)
        .map(|_| GS::from_generic_game_state(&GS::GenericGameState::generate_random_state_with_blocks_rng(&mut rng, block_count)))
        .collect();


    let mut tasks = Vec::with_capacity(number_random_states);

    for (i, state) in random_states.iter().enumerate() {
        let state_copy = state.clone();
        tasks.push(tokio::spawn(async move {
            let result = alpha_beta_sorted_minimax::<GS, 3>(&state_copy, depth);
            println!("State {}, evaluated states: {}", i, result.1);
            return result.1;
        }));
    }

    let mut evaluated_states = Vec::with_capacity(number_random_states);
    for task in tasks {
        evaluated_states.push(task.await?);
    }

    let evaluated_states_sum: usize = evaluated_states.iter().sum();
    let average_evaluated_states = evaluated_states_sum as f32 / number_random_states as f32;
    let average_branching_factor = average_evaluated_states.powf(1.0 / (depth as f32));
    println!("Average branching factor: {}", average_branching_factor);

    return Ok(average_branching_factor);
}

fn main() {
    let available_threads = std::thread::available_parallelism().expect("Could not get number of threads").get();
    // For benchmarking, it is better to leave some threads for other tasks so that tasks are less likely to be preempted
    let tokio_threads = if available_threads > 4 { available_threads - 4 } else { available_threads };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(tokio_threads)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
}

async fn async_main() {
    type GS5x5 = GameState5x5BinaryComposite;
    type GGS5x5 = <GameState5x5Struct as GameState>::GenericGameState;

    let benchmark_minimax_simple_5x5 = BenchmarkMinimaxSimple::<GS5x5>::new(
        "Santorini".to_string(),
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        6,
        1000,
        20,
    );
    benchmark_minimax_simple_5x5.gather_and_store_data().await.unwrap();
    //benchmark_minimax_simple_5x5.generate_graph_from_most_recent_data().unwrap();


    let benchmark_minimax_alpha_beta_5x5 = BenchmarkMinimaxAlphaBeta::new(
        "Santorini".to_string(),
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        8,
        1000,
        20,
        benchmark_minimax_simple_5x5,
    );
    benchmark_minimax_alpha_beta_5x5.gather_and_store_data().await.unwrap();
    //benchmark_minimax_alpha_beta_5x5.generate_graph_from_most_recent_data().unwrap();

    //let most_recent_data = benchmark_minimax_alpha_beta_5x5.get_most_recent_data_file().unwrap();
    //let data = benchmark_minimax_alpha_beta_5x5.get_data(&most_recent_data).unwrap();
    //let max_exec_time = data.raw_measurements_alpha_beta.iter().map(|m| m.iter().map(|m| m.computation_time.as_nanos()).max().unwrap()).max().unwrap();
    //println!("Max exec time: {}", max_exec_time);


    let benchmark_minimax_sorted_5x5_always_sort  = BenchmarkMinimaxSorted::<GS5x5, 0>::new(
        "Santorini".to_string(),
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        10,
        1000,
        20,
        benchmark_minimax_alpha_beta_5x5.clone(),
    );
    benchmark_minimax_sorted_5x5_always_sort.gather_and_store_data().await.unwrap();
    //benchmark_minimax_sorted_5x5_always_sort.generate_graph_from_most_recent_data().unwrap();


    let benchmark_minimax_sorted_5x5 = BenchmarkMinimaxSorted::<GS5x5, 3>::new(
        "Santorini".to_string(),
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        11,
        1000,
        20,
        benchmark_minimax_alpha_beta_5x5,
    );
    benchmark_minimax_sorted_5x5.gather_and_store_data().await.unwrap();
    //benchmark_minimax_sorted_5x5.generate_graph_from_most_recent_data().unwrap();


    let benchmark_minimax_cached_5x5 = BenchmarkMinimaxCached::<GS5x5, 3, 3, 3>::new(
        "Santorini".to_string(),
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        11,
        1000,
        20,
        benchmark_minimax_sorted_5x5,
    );
    benchmark_minimax_cached_5x5.gather_and_store_data().await.unwrap();
    //benchmark_minimax_cached_5x5.generate_graph_from_most_recent_data().unwrap();



    //let _ = average_branching_factor::<GS5x5>(1000, 20, 6).await;

    /*
    type GS5x5 = GameState5x5Struct;
    type GS4x4 = GameState4x4Binary3Bit;
    type GGS5x5 = <GameState5x5Struct as GameState>::GenericGameState;

    let branching_factor_stat = BranchingFactorByBlockCount::<GS4x4>::new("4x4".to_string(), " (4x4 Santorini)".to_string(), 60, 64, 1000000);

    //branching_factor_stat.gather_and_store_data().unwrap();
    branching_factor_stat.generate_graph_from_most_recent_data().unwrap();

    let game_states_stat = GameStatesByBlockCount::new(16, 1, " (4x4 Santorini)".to_string());
    let data = game_states_stat.gather_data().unwrap();
    let sum: u128 = data.game_states_by_block_count.iter().sum();
    println!("Sum: {}", sum);
    //game_states_stat.gather_and_store_data().unwrap();
    game_states_stat.generate_graph_from_most_recent_data().unwrap();

     */
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
