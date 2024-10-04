use crate::game_state::GameState;

pub trait Strategy : Copy + Clone + Send + Sync {
    type GameState: GameState;

    fn choose_move(&self, is_player_a: bool, current_state: &Self::GameState, possible_next_states: &Vec<Self::GameState>) -> usize;
}

pub mod heuristics;
pub mod random_strategy;
pub mod minimax_strategy_4x4;
pub mod console_input_strategy;
pub mod heuristic_minimax_strategy;