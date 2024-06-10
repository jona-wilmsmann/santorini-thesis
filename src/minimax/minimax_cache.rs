use crate::game_state::GameState;
use fnv::FnvHashMap;


pub struct MinimaxCache {
    pub evaluated_states: usize,
    pub pruned_states: usize,
    pub static_valuations: FnvHashMap<GameState, f32>,
}

impl MinimaxCache {
    pub fn new() -> MinimaxCache {
        return MinimaxCache {
            evaluated_states: 0,
            pruned_states: 0,
            static_valuations: FnvHashMap::default(),
        };
    }
}