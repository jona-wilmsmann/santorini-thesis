mod game_state;
mod minimax;

use std::time::Instant;
use crate::game_state::binary_3bit_game_state::Binary3BitGameState;
use crate::game_state::binary_4bit_game_state::Binary4BitGameState;
use crate::game_state::GameState;
use crate::game_state::generic_game_state::GenericGameState;

fn recursive_search(game_state: GameState, depth: u8) -> usize {
    let mut count = 0;
    if depth == 0 {
        return count;
    }

    let possible_next_states = game_state.get_possible_next_states();
    for state in possible_next_states {
        count += 1 + recursive_search(state.get_flipped_state(), depth - 1);
    }
    return count;
}

fn recursive_search4(game_state: Binary4BitGameState, depth: u8) -> usize {
    let mut count = 0;
    if depth == 0 {
        return count;
    }

    let possible_next_states = game_state.get_possible_next_states();
    for state in possible_next_states {
        count += 1 + recursive_search4(state.get_flipped_state(), depth - 1);
    }
    return count;
}

fn main() {
    let state = GenericGameState::new(1, 2, [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]).unwrap();

    let state_3b = Binary3BitGameState::from_generic_game_state(&state);
    let state_4b = Binary4BitGameState::from_generic_game_state(&state);

    println!("Normal 3b {}", state_3b);
    println!("Normal 4b {}", state_4b);

    let start_3b = Instant::now();
    let recursive_count_3b = recursive_search(state_3b, 4);
    let duration_3b = start_3b.elapsed();
    let start_4b = Instant::now();
    let recursive_count_4b = recursive_search4(state_4b, 4);
    let duration_4b = start_4b.elapsed();

    println!("Recursive count 3b: {}, 4b: {}", recursive_count_3b, recursive_count_4b);
    println!("Time elapsed 3b: {:?}, 4b: {:?}", duration_3b, duration_4b);

}
