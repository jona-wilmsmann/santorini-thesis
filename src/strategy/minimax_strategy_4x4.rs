use crate::game_state::GameState;
use crate::game_state::binary_3bit_game_state::Binary3BitGameState;
use crate::generic_game_state::generic_4x4_game_state::Generic4x4GameState;
use crate::minimax::minimax;
use crate::minimax::minimax_cache::MinimaxCache;
use crate::strategy::Strategy;

pub struct MinimaxStrategy4x4 {
    max_depth: usize,
    minimax_cache: MinimaxCache<Binary3BitGameState>
}

impl MinimaxStrategy4x4 {
    pub fn new(max_depth: usize) -> MinimaxStrategy4x4 {
        MinimaxStrategy4x4 {
            max_depth,
            minimax_cache: MinimaxCache::<Binary3BitGameState>::new()
        }
    }
}

impl Strategy for MinimaxStrategy4x4 {
    type GenericGameState = Generic4x4GameState;

    fn choose_move(&mut self, current_state: Generic4x4GameState, possible_next_states: Vec<Generic4x4GameState>) -> usize {

        let mut best_move_index = 0;
        let mut best_value = f32::NEG_INFINITY;

        for (i, generic_state) in possible_next_states.iter().enumerate() {
            let game_state = Binary3BitGameState::from_generic_game_state(generic_state);
            let value = minimax(&game_state, self.max_depth, f32::MIN, f32::MAX, &mut self.minimax_cache);

            if value > best_value {
                best_value = value;
                best_move_index = i;
            }
        }

        return best_move_index;
    }

    fn clear_cache(&mut self) {
        self.minimax_cache = MinimaxCache::<Binary3BitGameState>::new();
    }
}