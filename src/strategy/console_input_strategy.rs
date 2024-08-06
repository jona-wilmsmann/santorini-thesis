use crate::generic_game_state::GenericGameState;
use crate::strategy::Strategy;

pub struct ConsoleInputStrategy<GGS: GenericGameState> {
    _marker: std::marker::PhantomData<GGS>,
}

impl<GGS: GenericGameState> ConsoleInputStrategy<GGS> {
    pub fn new() -> ConsoleInputStrategy<GGS> {
        ConsoleInputStrategy { _marker: Default::default() }
    }

    fn get_user_input(max_value: usize) -> usize {
        println!("Enter a number between 0 and {}", max_value);

        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let index = input.trim().parse::<usize>();
            match index {
                Ok(index) => {
                    if index < max_value {
                        return index;
                    }
                },
                Err(_) => {
                    println!("Invalid input. Please enter a number between 0 and {}", max_value);
                }
            }
        }
    }
}

impl<GGS: GenericGameState> Strategy for ConsoleInputStrategy<GGS> {
    type GenericGameState = GGS;

    fn choose_move(&mut self, current_state: GGS, possible_next_states: Vec<GGS>) -> usize {
        println!("Current state:\n{}", current_state);
        println!("Possible next states:");
        for (i, state) in possible_next_states.iter().enumerate() {
            println!("Index {}:\n{}", i, state);
        }
        return Self::get_user_input(possible_next_states.len());
    }

    fn clear_cache(&mut self) {}
}