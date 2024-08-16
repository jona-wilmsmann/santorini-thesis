use rand::Rng;
use crate::generic_game_state::GenericGameState;
use crate::strategy::Strategy;

pub struct RandomStrategy<GGS: GenericGameState> {
    _marker: std::marker::PhantomData<GGS>,
}

impl<GGS: GenericGameState> RandomStrategy<GGS> {
    pub fn new() -> RandomStrategy<GGS> {
        RandomStrategy { _marker: Default::default() }
    }
}

impl<GGS: GenericGameState> Strategy for RandomStrategy<GGS> {
    type GenericGameState = GGS;

    fn choose_move(&mut self, _current_state: &GGS, possible_next_states: &Vec<GGS>) -> usize {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..possible_next_states.len());
        return index;
    }

    fn clear_cache(&mut self) {}
}