use std::ops::{Range, RangeInclusive};
use crate::game_state::GameState;
use crate::game_state::utils::random_state_generation::generate_random_state_with_blocks;
use crate::measurements::measure_minimax::measure_minimax;
use crate::measurements::minimax_measurement::MinimaxMeasurement;

pub async fn parallelize_measurements(random_state_amount: usize, block_amounts: RangeInclusive<usize>, depths: RangeInclusive<usize>) -> Vec<MinimaxMeasurement> {
    let mut handles = Vec::new();

    for block_amount in block_amounts {
        let random_generic_states = (0..random_state_amount)
            .map(|_| generate_random_state_with_blocks(block_amount)).collect::<Vec<_>>();
        let random_game_states = random_generic_states.iter()
            .map(|generic_state| GameState::from_generic_game_state(generic_state)).collect::<Vec<_>>();

        for (i, game_state) in random_game_states.iter().enumerate() {
            for depth in depths.clone() {
                let game_state_clone = game_state.clone();
                let handle = tokio::spawn(async move {
                    let measurement = measure_minimax(game_state_clone, block_amount, depth);
                    println!("Block {}: Finished measurement {}/{} with depth {}, took {:?}", block_amount, i + 1, random_state_amount, depth, measurement.calculation_time);
                    return measurement;
                });
                handles.push(handle);
            }
        }
    }

    let mut measurements = Vec::new();
    for handle in handles {
        measurements.push(handle.await.unwrap());
    }

    return measurements;
}