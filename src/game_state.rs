pub mod generic_game_state;

#[allow(dead_code)]
pub mod binary_4bit_game_state;
pub mod binary_3bit_game_state;
pub mod utils;

mod game_state_tests;

pub type GameState = binary_3bit_game_state::Binary3BitGameState;