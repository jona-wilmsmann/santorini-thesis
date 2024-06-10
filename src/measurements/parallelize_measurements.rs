use std::ops::Range;
use crate::game_state::GameState;
use crate::game_state::utils::random_state_generation::generate_random_state_with_blocks;
use crate::measurements::measure_minimax::measure_minimax;
use crate::measurements::minimax_measurement::MinimaxMeasurement;

pub async fn parallelize_measurements(random_state_amount: usize, block_amount: usize, depths: Range<usize>) -> Vec<MinimaxMeasurement> {
    let random_generic_states = (0..random_state_amount)
        .map(|_| generate_random_state_with_blocks(block_amount)).collect::<Vec<_>>();
    let random_game_states = random_generic_states.iter()
        .map(|generic_state| GameState::from_generic_game_state(generic_state)).collect::<Vec<_>>();

    let mut handles = Vec::new();

    for game_state in &random_game_states {
        for depth in depths.clone() {
            let game_state_clone = game_state.clone();
            let handle = tokio::spawn(async move {
                measure_minimax(game_state_clone, depth)
            });
            handles.push(handle);
        }
    }

    let mut measurements = Vec::new();
    for handle in handles {
        measurements.push(handle.await.unwrap());
    }

    return measurements;
}