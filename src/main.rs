// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]

use santorini_minimax::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use anyhow::Result;
use rand::SeedableRng;
use santorini_minimax::game_state::game_state_5x5_binary_composite::GameState5x5BinaryComposite;
use santorini_minimax::game_state::game_state_5x5_struct::GameState5x5Struct;
use santorini_minimax::game_state::{GameState, SantoriniEval, SantoriniState5x5};
use santorini_minimax::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use santorini_minimax::generic_game_state::GenericGameState;
use santorini_minimax::minimax::alpha_beta_sorted_minimax;
use santorini_minimax::play_game::simulate_random_games;
use santorini_minimax::stats::benchmark_minimax_alpha_beta::BenchmarkMinimaxAlphaBeta;
use santorini_minimax::stats::benchmark_minimax_cached::BenchmarkMinimaxCached;
use santorini_minimax::stats::benchmark_minimax_infinite::BenchmarkMinimaxInfinite;
use santorini_minimax::stats::benchmark_minimax_simple::BenchmarkMinimaxSimple;
use santorini_minimax::stats::benchmark_minimax_sorted::BenchmarkMinimaxSorted;
use santorini_minimax::stats::minimax_solve_stats::MinimaxSolveStats;
use santorini_minimax::stats::presolve_analysis::PresolveAnalysis;
use santorini_minimax::strategy::console_input_strategy::ConsoleInputStrategy;
use santorini_minimax::strategy::dynamic_minimax_strategy::DynamicMinimaxStrategy;
use santorini_minimax::strategy::heuristic_minimax_strategy::HeuristicMinimaxStrategy;
use santorini_minimax::strategy::heuristics::boreham_greedy_heuristic::boreham_greedy_heuristic;
use santorini_minimax::strategy::heuristics::boreham_heuristic::boreham_heuristic;
use santorini_minimax::strategy::heuristics::dynamic_heuristic::DynamicHeuristicParams;
use santorini_minimax::strategy::random_strategy::RandomStrategy;


fn store_game_state_image<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize>
(game_state: &GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER>, name: &str) -> Result<()> {
    let folder_path = "stats/game_states/";
    std::fs::create_dir_all(folder_path)?;
    let image_path = format!("{}/{}.svg", folder_path, name);
    game_state.draw_image(image_path.as_str())?;
    return Ok(());
}

async fn average_branching_factor<GS: GameState + SantoriniEval + 'static>(number_random_states: usize, block_count: usize, depth: usize) -> Result<f32> {
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

async fn handle_stats() {
    type GS5x5 = GameState5x5BinaryComposite;
    type GGS5x5 = <GameState5x5Struct as GameState>::GenericGameState;
    type GS4x4 = GameState4x4Binary3Bit;


    let benchmark_minimax_simple_5x5 = BenchmarkMinimaxSimple::<GS5x5>::new(
        "Santorini".to_string(),
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        6,
        1000,
        20,
    );
    //benchmark_minimax_simple_5x5.gather_and_store_data().await.unwrap();
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
    //benchmark_minimax_alpha_beta_5x5.gather_and_store_data().await.unwrap();
    //benchmark_minimax_alpha_beta_5x5.generate_graph_from_most_recent_data().unwrap();

    /*
    let most_recent_data = benchmark_minimax_alpha_beta_5x5.get_most_recent_data_file().unwrap();
    let data = benchmark_minimax_alpha_beta_5x5.get_data(&most_recent_data).unwrap();
    let max_exec_time = data.raw_measurements_alpha_beta.iter().map(|m| m.iter().filter(|m| m.result.is_finite()).map(|m| m.computation_time.as_nanos()).max().unwrap()).max().unwrap();
    println!("Max exec time: {}", max_exec_time);
     */


    let benchmark_minimax_sorted_5x5_always_sort = BenchmarkMinimaxSorted::<GS5x5, 0>::new(
        "Santorini".to_string(),
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        10,
        1000,
        20,
        benchmark_minimax_alpha_beta_5x5.clone(),
    );
    //benchmark_minimax_sorted_5x5_always_sort.gather_and_store_data().await.unwrap();
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
    //benchmark_minimax_sorted_5x5.gather_and_store_data().await.unwrap();
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
    //benchmark_minimax_cached_5x5.gather_and_store_data().await.unwrap();
    //benchmark_minimax_cached_5x5.generate_graph_from_most_recent_data().unwrap();


    let minimax_solve_stats_5x5 = MinimaxSolveStats::<GS5x5>::new(
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        1..=10,
        0..=92,
        1000,
    );
    //minimax_solve_stats_5x5.gather_and_store_data().await.unwrap();
    //minimax_solve_stats_5x5.generate_graph_from_most_recent_data().unwrap();

    /*
    let recent_data_file = minimax_solve_stats_5x5.get_most_recent_data_file().unwrap();
    let solve_data = minimax_solve_stats_5x5.get_data(&recent_data_file).unwrap();

    let states_by_block_count = GameStatesByBlockCount::new(25, 2);
    let block_data = states_by_block_count.gather_data().await.unwrap();

    let depth_1_data = solve_data.raw_measurements.iter().filter(|m| m.depth == 1);

    let mut approx_total_solved = 0u128;
    for block_count in 0..=92 {
        let solved_states = depth_1_data.clone().filter(|m| m.block_count == block_count).filter(|m| m.solved).count();
        let portion = solved_states as f64 / 1000.0;

        approx_total_solved += (block_data.game_states_by_block_count[block_count] * portion) as u128;
        println!("Block count: {}, solved states: {}, portion: {}", block_count, solved_states, portion);
    }

    println!("Approx total solved: {}", approx_total_solved);

     */

    /*
    let most_recent_data = minimax_solve_stats_5x5.get_most_recent_data_file().unwrap();
    let solve_data = minimax_solve_stats_5x5.get_data(&most_recent_data).unwrap();

    let game_states_by_block_count = GameStatesByBlockCount::new(25, 2, " (5x5 Binary Composite)".to_string());
    let block_data = game_states_by_block_count.gather_data().await.unwrap();

    let depth_11_data = solve_data.raw_measurements.iter().filter(|m| m.depth == 11);

    let mut total_states = 0;
    let mut total_solved_states = 0;
    for block_count in 0..=92 {
        let mut solved_states = depth_11_data.clone().filter(|m| m.block_count == block_count).filter(|m| m.solved).count();
        let portion = solved_states as f64 / 1000.0;

        println!("Block count: {}, solved states: {}, portion: {}", block_count, solved_states, portion);

        total_states += block_data.game_states_by_block_count[block_count];
        total_solved_states += (block_data.game_states_by_block_count[block_count] as f64 * portion) as u128;
    }

    println!("Total states: {}, total solved states: {}", total_states, total_solved_states);
    println!("Portion: {}", total_solved_states as f64 / total_states as f64);
     */


    let minimax_solve_stats_4x4 = MinimaxSolveStats::<GS4x4>::new(
        "4x4 Binary 3 Bit".to_string(),
        "4x4_binary_3_bit".to_string(),
        1..=13,
        0..=60,
        1000,
    );
    //minimax_solve_stats_4x4.gather_and_store_data().await.unwrap();
    //minimax_solve_stats_4x4.generate_graph_from_most_recent_data().unwrap();


    let presolve_analysis = PresolveAnalysis::new();
    //presolve_analysis.gather_and_store_data().await.unwrap();
    //presolve_analysis.generate_graph_from_most_recent_data().unwrap();


    let benchmark_minimax_infinite = BenchmarkMinimaxInfinite::<GS5x5>::new(
        "5x5 Binary Composite".to_string(),
        "5x5_binary_composite".to_string(),
        10000,
        57..=92,
    );

    //benchmark_minimax_infinite.gather_and_store_data().await.unwrap();
    //benchmark_minimax_infinite.generate_graph_from_most_recent_data().unwrap();
}

async fn benchmark_strategies<GS: GameState + SantoriniEval<SantoriniState = SantoriniState5x5> + 'static>(num_games: usize, initial_block_count: usize) {
    let boreham_greedy_strategy = HeuristicMinimaxStrategy::<GS>::new(0, boreham_greedy_heuristic);
    let boreham_strategy_depth_2 = HeuristicMinimaxStrategy::<GS>::new(2, boreham_heuristic);
    let boreham_strategy_depth_3 = HeuristicMinimaxStrategy::<GS>::new(3, boreham_heuristic);
    let boreham_strategy_depth_4 = HeuristicMinimaxStrategy::<GS>::new(4, boreham_heuristic);

    let child_heuristic_strategy_depth_2 = HeuristicMinimaxStrategy::<GS>::new(2, |state| state.get_child_evaluation());
    let child_heuristic_strategy_depth_3 = HeuristicMinimaxStrategy::<GS>::new(3, |state| state.get_child_evaluation());
    let child_heuristic_strategy_depth_4 = HeuristicMinimaxStrategy::<GS>::new(4, |state| state.get_child_evaluation());

    let random_strategy = RandomStrategy::<GS>::new();
    let console_input_strategy = ConsoleInputStrategy::<GS>::new();

    let default_params = DynamicHeuristicParams::default();
    let default_strategy_depth_2 = DynamicMinimaxStrategy::<GS>::new(2, default_params.clone());
    let default_strategy_depth_3 = DynamicMinimaxStrategy::<GS>::new(3, default_params.clone());
    let default_strategy_depth_4 = DynamicMinimaxStrategy::<GS>::new(4, default_params.clone());


    let start = std::time::Instant::now();
    let results = simulate_random_games(&default_strategy_depth_4, &child_heuristic_strategy_depth_4, num_games, initial_block_count).await;
    let duration = start.elapsed();

    println!("Wins: {}", results.strategy_1_wins);
    let average_turns = results.raw_games.iter().map(|g| g.num_turns).sum::<usize>() as f32 / num_games as f32;
    println!("Average turns: {}", average_turns);
    println!("Duration: {}s", duration.as_secs_f64());
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
    runtime.block_on(tokio_main());
}

async fn tokio_main() {
    type GS5x5 = GameState5x5BinaryComposite;
    type GGS5x5 = <GameState5x5Struct as GameState>::GenericGameState;
    type GS4x4 = GameState4x4Binary3Bit;

    //handle_stats().await;

    //benchmark_strategies::<GS5x5>(1000, 0).await;

    //let _ = average_branching_factor::<GS5x5>(1000, 20, 6).await;
}
