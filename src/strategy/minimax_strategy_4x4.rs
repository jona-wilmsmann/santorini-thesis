use crate::game_state::GameState;
use crate::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use crate::minimax::minimax;
use crate::minimax::minimax_cache::MinimaxCache;
use crate::strategy::Strategy;

pub struct MinimaxStrategy4x4 {
    max_depth: usize,
    minimax_cache: MinimaxCache<GameState4x4Binary3Bit>
}

impl MinimaxStrategy4x4 {
    pub fn new(max_depth: usize) -> MinimaxStrategy4x4 {
        MinimaxStrategy4x4 {
            max_depth,
            minimax_cache: MinimaxCache::<GameState4x4Binary3Bit>::new()
        }
    }
}

impl Strategy for MinimaxStrategy4x4 {
    type GenericGameState = GenericSantoriniGameState<4, 4, 1>;

    fn choose_move(&mut self, _current_state: &Self::GenericGameState, possible_next_states: &Vec<Self::GenericGameState>) -> usize {

        let mut best_move_index = 0;
        let mut best_value = f32::NEG_INFINITY;

        for (i, generic_state) in possible_next_states.iter().enumerate() {
            let game_state = GameState4x4Binary3Bit::from_generic_game_state(generic_state);
            let value = minimax(&game_state, self.max_depth, f32::MIN, f32::MAX, &mut self.minimax_cache);

            if value > best_value {
                best_value = value;
                best_move_index = i;
            }
        }

        return best_move_index;
    }

    fn clear_cache(&mut self) {
        self.minimax_cache = MinimaxCache::<GameState4x4Binary3Bit>::new();
    }
}