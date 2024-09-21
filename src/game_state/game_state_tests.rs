#[cfg(test)]
mod tests {
    use crate::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
    use crate::game_state::game_state_4x4_binary_4bit::GameState4x4Binary4Bit;
    use crate::game_state::game_state_4x4_struct::GameState4x4Struct;
    use crate::game_state::game_state_5x5_binary_128bit::GameState5x5Binary128bit;
    use crate::game_state::game_state_5x5_binary_composite::GameState5x5BinaryComposite;
    use crate::game_state::game_state_5x5_struct::GameState5x5Struct;
    use crate::game_state::{GameState, SimplifiedState};
    use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
    use crate::generic_game_state::GenericGameState;

    fn find_4x4_generic_discrepancies(tries: usize) {
        let mut states_to_test = Vec::with_capacity(tries + 2);

        let generic_state_with_no_workers = GenericSantoriniGameState::<4, 4, 1>::new(
            None,
            None,
            [[0; 4]; 4],
            true,
        ).unwrap();

        let generic_state_without_all_workers = GenericSantoriniGameState::<4, 4, 1>::new(
            Some([0]),
            None,
            [[0; 4]; 4],
            false,
        ).unwrap();

        states_to_test.push(generic_state_with_no_workers);
        states_to_test.push(generic_state_without_all_workers);


        for state_to_test in states_to_test {
            let state_3b = GameState4x4Binary3Bit::from_generic_game_state(&state_to_test);
            let converted_generic_state_3b = state_3b.to_generic_game_state();

            let state_4b = GameState4x4Binary4Bit::from_generic_game_state(&state_to_test);
            let converted_generic_state_4b = state_4b.to_generic_game_state();

            let state_struct = GameState4x4Struct::from_generic_game_state(&state_to_test);
            let converted_generic_state_struct = state_struct.to_generic_game_state();

            assert_eq!(state_to_test, converted_generic_state_3b);
            assert_eq!(state_to_test, converted_generic_state_4b);
            assert_eq!(state_to_test, converted_generic_state_struct);

            assert_eq!(state_3b.is_player_a_turn(), state_4b.is_player_a_turn());
            assert_eq!(state_3b.is_player_a_turn(), state_struct.is_player_a_turn());

            assert_eq!(state_3b.has_player_a_won(), state_4b.has_player_a_won());
            assert_eq!(state_3b.has_player_a_won(), state_struct.has_player_a_won());

            assert_eq!(state_3b.has_player_b_won(), state_4b.has_player_b_won());
            assert_eq!(state_3b.has_player_b_won(), state_struct.has_player_b_won());
        }
    }

    fn find_4x4_child_discrepancies(tries: usize) {
        let mut states_to_test = Vec::with_capacity(tries + 2);

        let generic_state_with_no_workers = GenericSantoriniGameState::<4, 4, 1>::new(
            None,
            None,
            [[0; 4]; 4],
            true,
        ).unwrap();

        let generic_state_without_all_workers = GenericSantoriniGameState::<4, 4, 1>::new(
            Some([0]),
            None,
            [[0; 4]; 4],
            false,
        ).unwrap();

        states_to_test.push(generic_state_with_no_workers);
        states_to_test.push(generic_state_without_all_workers);


        for state_to_test in states_to_test {
            let binary_3b_state = GameState4x4Binary3Bit::from_generic_game_state(&state_to_test);
            let binary_4b_state = GameState4x4Binary4Bit::from_generic_game_state(&state_to_test);
            let struct_state = GameState4x4Struct::from_generic_game_state(&state_to_test);

            let child_states_3b = binary_3b_state.get_children_states();
            let child_states_4b = binary_4b_state.get_children_states();
            let child_states_struct = struct_state.get_children_states();

            for child_state in &child_states_3b {
                let generic_child_state = child_state.to_generic_game_state();
                let back_converted_state = GameState4x4Binary3Bit::from_generic_game_state(&generic_child_state);
                assert_eq!(child_state.raw_value(), back_converted_state.raw_value());
            }
            for child_state in &child_states_4b {
                let generic_child_state = child_state.to_generic_game_state();
                let back_converted_state = GameState4x4Binary4Bit::from_generic_game_state(&generic_child_state);
                assert_eq!(child_state.raw_value(), back_converted_state.raw_value());
            }
            for child_state in &child_states_struct {
                let generic_child_state = child_state.to_generic_game_state();
                let back_converted_state = GameState4x4Struct::from_generic_game_state(&generic_child_state);
                assert_eq!(child_state.raw_value(), back_converted_state.raw_value());
            }

            let mut generic_child_states_3b = child_states_3b.iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<4, 4, 1>>>();
            let mut generic_child_states_4b = child_states_4b.iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<4, 4, 1>>>();
            let mut generic_child_states_struct = child_states_struct.iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<4, 4, 1>>>();

            generic_child_states_3b.sort();
            generic_child_states_4b.sort();
            generic_child_states_struct.sort();

            assert_eq!(generic_child_states_3b, generic_child_states_4b);
            assert_eq!(generic_child_states_3b, generic_child_states_struct);
        }
    }

    fn find_4x4_simplified_discrepancies(tries: usize) {
        let mut states_to_test = Vec::with_capacity(tries + 2);

        let generic_state_with_no_workers = GenericSantoriniGameState::<4, 4, 1>::new(
            None,
            None,
            [[0; 4]; 4],
            true,
        ).unwrap();

        let generic_state_without_all_workers = GenericSantoriniGameState::<4, 4, 1>::new(
            Some([0]),
            None,
            [[0; 4]; 4],
            false,
        ).unwrap();

        states_to_test.push(generic_state_with_no_workers);
        states_to_test.push(generic_state_without_all_workers);


        for state_to_test in states_to_test {
            let state_3b = GameState4x4Binary3Bit::from_generic_game_state(&state_to_test);
            let simplified_state_3b = state_3b.get_simplified_state();
            let generic_simplified_state_3b = simplified_state_3b.to_generic_game_state();

            let state_4b = GameState4x4Binary4Bit::from_generic_game_state(&state_to_test);
            let simplified_state_4b = state_4b.get_simplified_state();
            let generic_simplified_state_4b = simplified_state_4b.to_generic_game_state();

            assert_eq!(state_3b.is_simplified(), state_4b.is_simplified());

            assert_eq!(simplified_state_3b.is_simplified(), true);
            assert_eq!(simplified_state_4b.is_simplified(), true);

            assert_eq!(generic_simplified_state_3b, generic_simplified_state_4b);
        }
    }

    fn find_5x5_generic_discrepancies(tries: usize) {
        let mut states_to_test = Vec::with_capacity(tries + 2);

        let generic_state_with_no_workers = GenericSantoriniGameState::<5, 5, 2>::new(
            None,
            None,
            [[0; 5]; 5],
            true,
        ).unwrap();

        let generic_state_without_all_workers = GenericSantoriniGameState::<5, 5, 2>::new(
            Some([0, 1]),
            None,
            [[0; 5]; 5],
            false,
        ).unwrap();

        states_to_test.push(generic_state_with_no_workers);
        states_to_test.push(generic_state_without_all_workers);


        for state_to_test in states_to_test {
            let binary_state = GameState5x5Binary128bit::from_generic_game_state(&state_to_test);
            let converted_generic_state_binary = binary_state.to_generic_game_state();

            let struct_state = GameState5x5Struct::from_generic_game_state(&state_to_test);
            let converted_generic_state_struct = struct_state.to_generic_game_state();

            let binary_2_state = GameState5x5BinaryComposite::from_generic_game_state(&state_to_test);
            let converted_generic_state_binary_2 = binary_2_state.to_generic_game_state();

            assert_eq!(state_to_test, converted_generic_state_binary);
            assert_eq!(state_to_test, converted_generic_state_struct);
            assert_eq!(state_to_test, converted_generic_state_binary_2);

            assert_eq!(binary_state.is_player_a_turn(), struct_state.is_player_a_turn());
            assert_eq!(binary_state.is_player_a_turn(), binary_2_state.is_player_a_turn());

            assert_eq!(binary_state.has_player_a_won(), struct_state.has_player_a_won());
            assert_eq!(binary_state.has_player_a_won(), binary_2_state.has_player_a_won());

            assert_eq!(binary_state.has_player_b_won(), struct_state.has_player_b_won());
            assert_eq!(binary_state.has_player_b_won(), binary_2_state.has_player_b_won());
        }
    }

    fn find_5x5_child_discrepancies(tries: usize) {
        let mut states_to_test = Vec::with_capacity(tries + 2);

        let generic_state_with_no_workers = GenericSantoriniGameState::<5, 5, 2>::new(
            None,
            None,
            [[0; 5]; 5],
            true,
        ).unwrap();

        let generic_state_without_all_workers = GenericSantoriniGameState::<5, 5, 2>::new(
            Some([0, 1]),
            None,
            [[0; 5]; 5],
            false,
        ).unwrap();

        states_to_test.push(generic_state_with_no_workers);
        states_to_test.push(generic_state_without_all_workers);

        for _ in 0..tries {
            states_to_test.push(GenericSantoriniGameState::<5, 5, 2>::generate_random_state());
        }

        for state_to_test in &states_to_test {
            let binary_state = GameState5x5Binary128bit::from_generic_game_state(state_to_test);
            let struct_state = GameState5x5Struct::from_generic_game_state(state_to_test);
            let binary_2_state = GameState5x5BinaryComposite::from_generic_game_state(state_to_test);

            let child_states_binary = binary_state.get_children_states();
            let child_states_struct = struct_state.get_children_states();
            let child_states_binary_2 = binary_2_state.get_children_states();

            for child_state in &child_states_binary {
                let generic_child_state = child_state.to_generic_game_state();
                let back_converted_state = GameState5x5Binary128bit::from_generic_game_state(&generic_child_state);
                assert_eq!(child_state.raw_value(), back_converted_state.raw_value());
            }
            for child_state in &child_states_struct {
                let generic_child_state = child_state.to_generic_game_state();
                let back_converted_state = GameState5x5Struct::from_generic_game_state(&generic_child_state);
                assert_eq!(child_state.raw_value(), back_converted_state.raw_value());
            }
            for child_state in &child_states_binary_2 {
                let generic_child_state = child_state.to_generic_game_state();
                let back_converted_state = GameState5x5BinaryComposite::from_generic_game_state(&generic_child_state);
                assert_eq!(child_state.raw_value(), back_converted_state.raw_value());
            }

            let mut generic_child_states_binary = child_states_binary.iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<5, 5, 2>>>();
            let mut generic_child_states_struct = child_states_struct.iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<5, 5, 2>>>();
            let mut generic_child_states_binary_2 = child_states_binary_2.iter().map(|state| state.to_generic_game_state()).collect::<Vec<GenericSantoriniGameState<5, 5, 2>>>();

            generic_child_states_binary.sort();
            generic_child_states_struct.sort();
            generic_child_states_binary_2.sort();

            assert_eq!(generic_child_states_binary, generic_child_states_struct);
            assert_eq!(generic_child_states_binary, generic_child_states_binary_2);
        }
    }

    #[test]
    fn test_find_4x4_generic_discrepancies() {
        find_4x4_generic_discrepancies(100000);
    }

    #[test]
    fn test_find_4x4_child_discrepancies() {
        find_4x4_child_discrepancies(100000);
    }

    #[test]
    fn test_find_4x4_simplified_discrepancies() {
        find_4x4_simplified_discrepancies(100000);
    }

    #[test]
    fn test_find_5x5_generic_discrepancies() {
        find_5x5_generic_discrepancies(10000);
    }

    #[test]
    fn test_find_5x5_child_discrepancies() {
        find_5x5_child_discrepancies(10000);
    }
}