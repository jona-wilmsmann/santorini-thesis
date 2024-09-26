use std::array::from_fn;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use crate::game_state::GameState;
use fnv::FnvHashMap;

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub value: f32,
    pub alpha: f32,
    pub beta: f32,
}

pub struct MinimaxCache<GS: GameState, const DEPTH_COUNT: usize> {
    pub valuation_bounds: [FnvHashMap<GS, Bounds>; DEPTH_COUNT],
}

impl<GS: GameState, const DEPTH_COUNT: usize> MinimaxCache<GS, DEPTH_COUNT> {
    pub fn new() -> MinimaxCache<GS, DEPTH_COUNT> {
        return MinimaxCache {
            valuation_bounds: from_fn(|_| FnvHashMap::default()),
        };
    }

    #[inline(always)]
    pub fn get_valuation_bounds(&self, depth: usize, game_state: &GS) -> Option<&Bounds> {
        return self.valuation_bounds[depth].get(game_state);
    }

    pub fn insert_valuation_bounds(&mut self, depth: usize, game_state: GS, bounds: Bounds) {
        //self.valuation_bounds[depth].insert(game_state, bounds);

        // TODO: Check if this makes sense and if there are any other possible cases to cover
        let entry = self.valuation_bounds[depth].entry(game_state);
        match entry {
            Occupied(mut occupied_entry) => {
                let current_bounds = occupied_entry.get_mut();
                if bounds.value > current_bounds.value {
                    *current_bounds = bounds;
                } else if bounds.value == current_bounds.value {
                    if bounds.alpha > current_bounds.alpha && bounds.beta <= current_bounds.beta {
                        current_bounds.alpha = bounds.alpha;
                    }
                    if bounds.beta < current_bounds.beta && bounds.alpha >= current_bounds.alpha {
                        current_bounds.beta = bounds.beta;
                    }
                }
            },
            Vacant(vacant_entry) => {
                vacant_entry.insert(bounds);
            }
        }
    }
}