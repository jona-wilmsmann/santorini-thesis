use crate::game_state::GameState;
use crate::measurements::minimax_measurement::MinimaxMeasurement;
use crate::minimax::minimax;
use crate::minimax::minimax_cache::MinimaxCache;

pub fn measure_minimax(game_state: GameState, depth: usize) -> MinimaxMeasurement {
    let mut cache = MinimaxCache::new();
    let start_time = std::time::Instant::now();
    let result = minimax(&game_state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut cache);
    let calculation_time = start_time.elapsed();

    return MinimaxMeasurement {
        game_state,
        depth,
        result,
        calculation_time,
        evaluated_states: cache.evaluated_states,
        pruned_states: cache.pruned_states,
    }
}