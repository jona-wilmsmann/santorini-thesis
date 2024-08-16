use crate::game_state::GameState;
use crate::strategy::Strategy;

// Returns true if player 1 wins, false if player 2 wins
// Player 1 is the first player to move
pub fn play_game<
    S1: Strategy<GenericGameState=GS::GenericGameState>,
    S2: Strategy<GenericGameState=GS::GenericGameState>,
    GS: GameState
>(player_1_strategy: &mut S1, player_2_strategy: &mut S2, initial_game_state: GS) -> bool {
    let mut current_game_state = initial_game_state;
    let mut player_1s_turn = true;

    loop {
        if current_game_state.has_player_a_won() {
            return player_1s_turn;
        }
        if current_game_state.has_player_b_won() {
            return !player_1s_turn;
        }

        let possible_next_states = current_game_state.get_children_states();
        if possible_next_states.len() == 0 {
            return !player_1s_turn;
        }

        let chosen_move = if player_1s_turn {
            player_1_strategy.choose_move(&current_game_state.to_generic_game_state(), &possible_next_states.iter().map(|state| state.to_generic_game_state()).collect())
        } else {
            player_2_strategy.choose_move(&current_game_state.to_generic_game_state(), &possible_next_states.iter().map(|state| state.to_generic_game_state()).collect())
        };

        current_game_state = possible_next_states[chosen_move].clone().get_flipped_state();
        player_1s_turn = !player_1s_turn;
    }
}