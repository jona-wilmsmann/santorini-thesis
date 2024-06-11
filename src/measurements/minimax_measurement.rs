use crate::game_state::GameState;

pub struct MinimaxMeasurement {
    pub game_state: GameState,
    pub game_state_block_amount: usize,
    pub depth: usize,
    pub result: f32,
    pub calculation_time: std::time::Duration,
    pub evaluated_states: usize,
    pub pruned_states: usize,
}