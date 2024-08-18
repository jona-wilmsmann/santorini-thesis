#[cfg(test)]
mod tests {
    use crate::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
    use crate::game_state::game_state_4x4_binary_4bit::GameState4x4Binary4Bit;
    use crate::game_state::game_state_5x5_binary_128bit::GameState5x5Binary128bit;
    use crate::game_state::GameState;
    use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
    use crate::generic_game_state::GenericGameState;

    fn find_4x4_discrepancies(tries: usize) {
        for _ in 0..tries {
            let mut random_state = GenericSantoriniGameState::<4, 4, 1>::generate_random_state();
            // Ensure player A turn is true, because the 4b states do not have player turn information
            random_state.set_player_a_turn(true);

            let binary_3b_state = GameState4x4Binary3Bit::from_generic_game_state(&random_state);
            let binary_4b_state = GameState4x4Binary4Bit::from_generic_game_state(&random_state);
            let mut next_states_3b = binary_3b_state.get_children_states().iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<4, 4, 1>>>();
            let mut next_states_4b = binary_4b_state.get_children_states().iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<4, 4, 1>>>();

            // Set player A turn to true for all states, because the 4b states do not have player turn information
            for state in next_states_3b.iter_mut() {
                state.set_player_a_turn(true);
            }

            next_states_3b.sort();
            next_states_4b.sort();

            assert_eq!(next_states_3b, next_states_4b);
        }
    }

    fn find_4x4_flip_discrepancies(tries: usize) {
        for _ in 0..tries {
            let random_state = GenericSantoriniGameState::<4, 4, 1>::generate_random_state();
            let binary_3b_state = GameState4x4Binary3Bit::from_generic_game_state(&random_state);
            let binary_4b_state = GameState4x4Binary4Bit::from_generic_game_state(&random_state);

            let flipped_3b = binary_3b_state.get_flipped_state();
            let flipped_4b = binary_4b_state.get_flipped_state();

            let flipped_3b_generic = flipped_3b.to_generic_game_state();
            let mut flipped_4b_generic = flipped_4b.to_generic_game_state();

            let twice_flipped_3b = flipped_3b.get_flipped_state();
            let twice_flipped_4b = flipped_4b.get_flipped_state();

            let twice_flipped_3b_generic = twice_flipped_3b.to_generic_game_state();
            let mut twice_flipped_4b_generic = twice_flipped_4b.to_generic_game_state();

            flipped_4b_generic.set_player_a_turn(flipped_3b_generic.player_a_turn);
            assert_eq!(flipped_3b_generic, flipped_4b_generic);

            assert_eq!(random_state, twice_flipped_3b_generic);

            twice_flipped_4b_generic.set_player_a_turn(random_state.player_a_turn);
            assert_eq!(random_state, twice_flipped_4b_generic);
        }
    }

    fn find_5x5_generic_conversion_discrepancies(tries: usize) {
        let generic_state_without_all_workers = GenericSantoriniGameState::<5, 5, 2>::new(
            [0, GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED],
            [1, GenericSantoriniGameState::<5, 5, 2>::WORKER_NOT_PLACED],
            [[0; 5]; 5],
            true
        ).unwrap();

        let binary_state_without_all_workers = GameState5x5Binary128bit::from_generic_game_state(&generic_state_without_all_workers);
        let converted_generic_state_without_all_workers = binary_state_without_all_workers.to_generic_game_state();

        assert_eq!(generic_state_without_all_workers, converted_generic_state_without_all_workers);

        for _ in 0..tries {
            let random_generic_state = GenericSantoriniGameState::<5, 5, 2>::generate_random_state();

            let binary_state = GameState5x5Binary128bit::from_generic_game_state(&random_generic_state);

            let converted_generic_state = binary_state.to_generic_game_state();

            assert_eq!(random_generic_state, converted_generic_state);
        }
    }

    #[test]
    fn test_find_discrepancies() {
        find_4x4_discrepancies(100000);
    }

    #[test]
    fn test_find_flip_discrepancies() {
        find_4x4_flip_discrepancies(100000);
    }

    #[test]
    fn test_find_5x5_generic_conversion_discrepancies() {
        find_5x5_generic_conversion_discrepancies(100000);
    }
}