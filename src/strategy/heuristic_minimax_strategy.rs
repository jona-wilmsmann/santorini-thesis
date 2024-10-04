use rand::Rng;
use crate::game_state::{GameState, SantoriniEval, SantoriniState5x5};
use crate::minimax::minimax_cache::MinimaxCache;
use crate::minimax::minimax_custom_heuristic;
use crate::strategy::Strategy;

#[derive(Copy, Clone)]
pub struct HeuristicMinimaxStrategy<GS: GameState + SantoriniEval<SantoriniState = SantoriniState5x5>> {
    max_depth: usize,
    heuristic_function: fn(&GS) -> f32,
    _marker: std::marker::PhantomData<GS>,
}

impl<GS: GameState + SantoriniEval<SantoriniState = SantoriniState5x5>> HeuristicMinimaxStrategy<GS> {
    pub fn new(max_depth: usize, heuristic_function: fn(&GS) -> f32) -> HeuristicMinimaxStrategy<GS> {
        assert!(max_depth < 100);
        return HeuristicMinimaxStrategy {
            max_depth,
            heuristic_function,
            _marker: std::marker::PhantomData,
        };
    }
}

impl<GS: GameState + SantoriniEval<SantoriniState = SantoriniState5x5>> Strategy for HeuristicMinimaxStrategy<GS> {
    type GameState = GS;

    fn choose_move(&self, is_player_a: bool, _current_state: &GS, possible_next_states: &Vec<GS>) -> usize {
        let mut cache = MinimaxCache::new();

        let mut best_move_indices = Vec::new();

        if is_player_a {
            let mut best_value = f32::NEG_INFINITY;

            for (i, game_state) in possible_next_states.iter().enumerate() {
                let value = minimax_custom_heuristic(game_state, self.max_depth, &mut cache, self.heuristic_function);

                if value > best_value {
                    best_value = value;
                    best_move_indices.clear();
                    best_move_indices.push(i);
                } else if value == best_value {
                    best_move_indices.push(i);
                }
            }
        } else {
            let mut worst_value = f32::INFINITY;

            for (i, game_state) in possible_next_states.iter().enumerate() {
                let value = minimax_custom_heuristic(game_state, self.max_depth, &mut cache, self.heuristic_function);

                if value < worst_value {
                    worst_value = value;
                    best_move_indices.clear();
                    best_move_indices.push(i);
                } else if value == worst_value {
                    best_move_indices.push(i);
                }
            }
        }

        return if best_move_indices.len() == 1 {
            best_move_indices[0]
        } else {
            let mut rng = rand::thread_rng();
            let best_move_index = rng.gen_range(0..best_move_indices.len());
            best_move_indices[best_move_index]
        }
    }
}