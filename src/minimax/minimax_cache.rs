use std::array::from_fn;
use crate::game_state::GameState;
use fnv::FnvHashMap;

pub struct Bounds {
    pub value: f32,
    pub alpha: f32,
    pub beta: f32,
}

pub struct MinimaxCache {
    pub evaluated_states: usize,
    pub pruned_states: usize,
    pub valuations: [FnvHashMap<GameState, Bounds>; 63],
}

impl MinimaxCache {
    pub fn new() -> MinimaxCache {
        return MinimaxCache {
            evaluated_states: 0,
            pruned_states: 0,
            valuations: from_fn(|_| FnvHashMap::default()),
        };
    }
}