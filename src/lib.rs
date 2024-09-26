// Necessary for precomputing values for static evaluation
#![feature(const_fn_floating_point_arithmetic)]

pub mod generic_game_state;
pub mod game_state;
pub mod minimax;
pub mod precompute_state_winner;
pub mod strategy;
pub mod play_game;
pub mod stats;