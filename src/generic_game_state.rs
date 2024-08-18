pub trait GenericGameState : std::fmt::Display + std::fmt::Debug {
    fn generate_random_state() -> Self;
    fn generate_random_state_rng<RNG: rand::Rng>(rng: &mut RNG) -> Self;
    fn generate_random_state_with_blocks(block_amount: usize) -> Self;
    fn generate_random_state_with_blocks_rng<RNG: rand::Rng>(rng: &mut RNG, block_amount: usize) -> Self;
}

pub mod generic_4x4_game_state;
pub mod generic_santorini_game_state;