#[cfg(test)]
mod tests {
    use crate::game_state::binary_3bit_game_state::Binary3BitGameState;
    use crate::game_state::binary_4bit_game_state::Binary4BitGameState;
    use crate::game_state::generic_game_state::GenericGameState;
    use crate::game_state::utils::random_state_generation::generate_random_state;

    fn find_discrepancies(tries: usize) {
        for _ in 0..tries {
            let random_state = generate_random_state();
            let binary_3b_state = Binary3BitGameState::from_generic_game_state(&random_state);
            let binary_4b_state = Binary4BitGameState::from_generic_game_state(&random_state);
            let mut next_states_3b = binary_3b_state.get_children_states().iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericGameState>>();
            let mut next_states_4b = binary_4b_state.get_children_states().iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericGameState>>();
            next_states_3b.sort();
            next_states_4b.sort();

            assert_eq!(next_states_3b, next_states_4b);
        }
    }

    fn find_flip_discrepancies(tries: usize) {
        for _ in 0..tries {
            let random_state = generate_random_state();
            let binary_3b_state = Binary3BitGameState::from_generic_game_state(&random_state);
            let binary_4b_state = Binary4BitGameState::from_generic_game_state(&random_state);

            let flipped_3b = binary_3b_state.get_flipped_state();
            let flipped_4b = binary_4b_state.get_flipped_state();

            let flipped_3b_generic = flipped_3b.to_generic_game_state();
            let flipped_4b_generic = flipped_4b.to_generic_game_state();

            let twice_flipped_3b = flipped_3b.get_flipped_state();
            let twice_flipped_4b = flipped_4b.get_flipped_state();

            let twice_flipped_3b_generic = twice_flipped_3b.to_generic_game_state();
            let twice_flipped_4b_generic = twice_flipped_4b.to_generic_game_state();

            assert_eq!(flipped_3b_generic, flipped_4b_generic);
            assert_eq!(random_state, twice_flipped_3b_generic);
            assert_eq!(random_state, twice_flipped_4b_generic);
        }
    }

    #[test]
    fn test_find_discrepancies() {
        find_discrepancies(100000);
    }

    #[test]
    fn test_find_flip_discrepancies() {
        find_flip_discrepancies(100000);
    }
}