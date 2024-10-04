use rand::Rng;
use crate::game_state::GameState;
use crate::strategy::Strategy;

#[derive(Copy, Clone)]
pub struct RandomStrategy<GS: GameState> {
    _marker: std::marker::PhantomData<GS>,
}

impl<GS: GameState> RandomStrategy<GS> {
    pub fn new() -> RandomStrategy<GS> {
        RandomStrategy {
            _marker: Default::default(),
        }
    }
}

impl<GS: GameState> Strategy for RandomStrategy<GS> {
    type GameState = GS;

    fn choose_move(&self, _is_player_a: bool, _current_state: &GS, possible_next_states: &Vec<GS>) -> usize {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..possible_next_states.len());
        return index;
    }
}