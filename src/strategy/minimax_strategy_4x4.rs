use crate::game_state::{GameState, SantoriniEval};
use crate::minimax::minimax;
use crate::minimax::minimax_cache::MinimaxCache;
use crate::strategy::Strategy;

#[derive(Copy, Clone)]
pub struct MinimaxStrategy<GS: GameState + SantoriniEval> {
    max_depth: usize,
    _marker: std::marker::PhantomData<GS>,
}

impl<GS: GameState + SantoriniEval> MinimaxStrategy<GS> {
    pub fn new(max_depth: usize) -> MinimaxStrategy<GS> {
        assert!(max_depth < 100);
        return MinimaxStrategy {
            max_depth,
            _marker: std::marker::PhantomData,
        };
    }
}

impl<GS: GameState + SantoriniEval> Strategy for MinimaxStrategy<GS> {
    type GameState = GS;

    fn choose_move(&self, is_player_a: bool, _current_state: &GS, possible_next_states: &Vec<GS>) -> usize {
        let mut cache = MinimaxCache::new();

        let mut best_move_index = 0;

        if is_player_a {
            let mut best_value = f32::NEG_INFINITY;

            for (i, game_state) in possible_next_states.iter().enumerate() {
                let value = minimax(game_state, self.max_depth, f32::MIN, f32::MAX, &mut cache);

                if value > best_value {
                    best_value = value;
                    best_move_index = i;
                }
            }
        } else {
            let mut worst_value = f32::INFINITY;

            for (i, game_state) in possible_next_states.iter().enumerate() {
                let value = minimax(game_state, self.max_depth, f32::MIN, f32::MAX, &mut cache);

                if value < worst_value {
                    worst_value = value;
                    best_move_index = i;
                }
            }
        }

        return best_move_index;
    }
}