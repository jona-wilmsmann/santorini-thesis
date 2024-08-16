use crate::generic_game_state::GenericGameState;

pub trait Strategy {
    type GenericGameState: GenericGameState;

    fn choose_move(&mut self, current_state: &Self::GenericGameState, possible_next_states: &Vec<Self::GenericGameState>) -> usize;

    fn clear_cache(&mut self);
}

pub mod random_strategy;
pub mod minimax_strategy_4x4;
pub mod console_input_strategy;