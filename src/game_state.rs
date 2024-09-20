use std::fmt::{Debug, Display};
use crate::generic_game_state::GenericGameState;
use crate::minimax::minimax_cache::MinimaxCache;

pub trait GameState: Display + Send + Copy + Clone + Eq + PartialEq + std::hash::Hash {
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

pub trait MinimaxReady : GameState {
    fn sort_children_states(children_states: &mut Vec<Self>, depth: usize, cache: &mut MinimaxCache<Self>);
    fn get_static_evaluation(&self) -> f32;
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
    fn get_block_count(&self) -> u64;
    fn get_continuous_block_id_count(block_count: usize) -> u64;
    fn get_continuous_block_id(&self) -> u64;
    fn from_continuous_block_id(block_count: usize, continuous_id: u64) -> Self;
}

#[allow(dead_code)]
pub mod game_state_4x4_binary_4bit;
pub mod game_state_4x4_binary_3bit;
pub mod game_state_5x5_binary_128bit;
pub mod game_state_5x5_struct;
pub mod game_state_5x5_binary_composite;
pub mod utils;

mod game_state_tests;