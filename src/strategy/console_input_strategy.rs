use crate::game_state::GameState;
use crate::strategy::Strategy;

#[derive(Copy, Clone)]
pub struct ConsoleInputStrategy<GS: GameState> {
    _marker: std::marker::PhantomData<GS>,
}

impl<GS: GameState> ConsoleInputStrategy<GS> {
    pub fn new() -> ConsoleInputStrategy<GS> {
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

impl<GS: GameState> Strategy for ConsoleInputStrategy<GS> {
    type GameState = GS;

    fn choose_move(&self, _is_player_a: bool, current_state: &GS, possible_next_states: &Vec<GS>) -> usize {
        println!("Current state:\n{}", current_state);
        println!("Possible next states:");
        for (i, state) in possible_next_states.iter().enumerate() {
            println!("Index {}:\n{}", i, state);
        }
        return Self::get_user_input(possible_next_states.len());
    }
}