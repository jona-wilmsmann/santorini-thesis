use std::sync::Arc;
use std::time::{Duration, Instant};
use rand::SeedableRng;
use tokio::sync::Mutex;
use crate::game_state::GameState;
use crate::generic_game_state::GenericGameState;
use crate::stats::utils::draw_minimax_benchmark::{AverageMinimaxMeasurement, MinimaxMeasurement};

pub async fn gather_minimax_benchmark<GS: GameState + 'static>(
    number_of_game_states: usize,
    block_count: usize,
    max_depth: usize,
    function: fn(state: &GS, depth: usize) -> (f32, usize),
) -> anyhow::Result<(Vec<Vec<MinimaxMeasurement>>, Vec<AverageMinimaxMeasurement>)> {
    if cfg!(debug_assertions) {
        return Err(anyhow::anyhow!("Benchmarking should be done in release mode"));
    }

    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let random_states: Vec<GS> = (0..number_of_game_states)
        .map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_with_blocks_rng(&mut rng, block_count))).collect();

    let mut tasks = Vec::with_capacity(number_of_game_states);
    let states_progress = Arc::new(Mutex::new(0));

    for state in random_states.into_iter() {
        let states_progress = states_progress.clone();
        let total_states = number_of_game_states;

        tasks.push(tokio::spawn(async move {
            let mut measurements = Vec::new();
            for depth in 0..=max_depth {
                let start = Instant::now();
                let (result, evaluated_states) = function(&state, depth);
                let computation_time = start.elapsed();
                let average_branching_factor = (evaluated_states as f32).powf(1.0 / (depth as f32));
                measurements.push(MinimaxMeasurement {
                    depth,
                    computation_time,
                    evaluated_states,
                    average_branching_factor,
                    result,
                });
            }
            let mut states_progress_lock = states_progress.lock().await;
            *states_progress_lock += 1;
            println!("Progress: {}/{}", *states_progress_lock, total_states);
            drop(states_progress_lock);
            return measurements;
        }));
    }

    let mut measurements = Vec::with_capacity(number_of_game_states);
    for task in tasks.into_iter() {
        measurements.push(task.await?);
    }

    let mut average_measurements = Vec::with_capacity(max_depth + 1);

    for depth in 0..=max_depth {
        let mut total_computation_time = Duration::new(0, 0);
        let mut total_evaluated_states = 0;
        let mut solved_states = 0;

        for single_measurement in measurements.iter() {
            total_computation_time += single_measurement[depth].computation_time;
            total_evaluated_states += single_measurement[depth].evaluated_states;
            if single_measurement[depth].result != 0.0 {
                solved_states += 1;
            }
        }

        average_measurements.push(AverageMinimaxMeasurement {
            depth,
            computation_time: total_computation_time / number_of_game_states as u32,
            evaluated_states: total_evaluated_states / number_of_game_states,
            average_branching_factor: ((total_evaluated_states / number_of_game_states) as f32).powf(1.0 / (depth as f32)),
            solve_portion: solved_states as f32 / number_of_game_states as f32,
        });
    }

    return Ok((measurements, average_measurements));
}