use std::fmt::{Debug, Display};
use crate::generic_game_state::GenericGameState;

pub struct SantoriniState4x4 {
    pub position_heights: [u8; 16],
    pub worker_a_position: u8,
    pub worker_b_position: u8,
    pub player_a_turn: bool,
}

pub struct SantoriniState5x5 {
    pub tile_heights: [u8; 25],
    pub worker_a_tiles: [u8; 2],
    pub worker_b_tiles: [u8; 2],
    pub player_a_turn: bool,
}

pub trait GameState: Display + Send + Sync + Copy + Clone + Eq + PartialEq + std::hash::Hash {
    type RawValue: Debug;
    type GenericGameState: GenericGameState;

    fn new(value: Self::RawValue) -> Self;
    fn raw_value(&self) -> Self::RawValue;
    fn is_player_a_turn(&self) -> bool;
    fn has_player_a_won(&self) -> bool;
    fn has_player_b_won(&self) -> bool;
    fn from_generic_game_state(generic_game_state: &Self::GenericGameState) -> Self;
    fn to_generic_game_state(&self) -> Self::GenericGameState;
    fn get_children_states(&self) -> Vec<Self>;
    fn get_children_states_reuse_vec(&self, possible_next_states: &mut Vec<Self>);
}

pub trait SantoriniEval: GameState {
    type SantoriniState;
    fn get_santorini_state(&self) -> Self::SantoriniState;
    fn get_child_evaluation(&self) -> f32;
}

pub trait SimplifiedState {
    fn get_simplified_state(&self) -> Self;
    fn is_simplified(&self) -> bool;
}

pub trait ContinuousId {
    fn get_continuous_id_count() -> u64;
    fn get_continuous_id(&self) -> u64;
    fn from_continuous_id(id: u64) -> Self;
}

pub trait ContinuousBlockId : SimplifiedState {
    fn get_block_count(&self) -> i64;
    fn get_continuous_block_id_count(block_count: isize) -> u64;
    fn get_continuous_block_id(&self) -> u64;
    fn from_continuous_block_id(block_count: isize, continuous_id: u64) -> Self;
}

#[allow(dead_code)]
pub mod game_state_4x4_binary_4bit;
pub mod game_state_4x4_binary_3bit;
pub mod game_state_4x4_struct;
pub mod game_state_5x5_binary_128bit;
pub mod game_state_5x5_struct;
pub mod game_state_5x5_binary_composite;
pub mod game_state_5x5_5bit;
pub mod utils;

mod game_state_tests;