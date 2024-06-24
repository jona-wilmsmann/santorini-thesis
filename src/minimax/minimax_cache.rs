use crate::game_state::GameState;
use fnv::FnvHashMap;


pub struct MinimaxCache {
    pub evaluated_states: usize,
    pub pruned_states: usize,
    pub static_valuations: FnvHashMap<GameState, f32>,
    pub best_child: FnvHashMap<GameState, GameState>,
}

impl MinimaxCache {
    pub fn new() -> MinimaxCache {
        return MinimaxCache {
            evaluated_states: 0,
            pruned_states: 0,
            static_valuations: FnvHashMap::default(),
            best_child: FnvHashMap::default(),
        };
    }
}