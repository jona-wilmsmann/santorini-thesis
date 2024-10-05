use crate::game_state::GameState;
use crate::generic_game_state::GenericGameState;
use crate::strategy::Strategy;

pub struct GameResult {
    pub player_a_wins: bool,
    pub num_turns: usize,
}

// Returns true if player A wins, false if player B wins
pub fn play_game<
    S1: Strategy<GameState=GS>,
    S2: Strategy<GameState=GS>,
    GS: GameState
>(player_a_strategy: &S1, player_b_strategy: &S2, initial_game_state: GS) -> GameResult {
    let mut current_game_state = initial_game_state;
    let mut player_a_turn = initial_game_state.is_player_a_turn();

    let mut num_turns = 0;
    loop {
        if current_game_state.has_player_a_won() {
            return GameResult {
                player_a_wins: true,
                num_turns,
            };
        }
        if current_game_state.has_player_b_won() {
            return GameResult {
                player_a_wins: false,
                num_turns,
            };
        }

        let possible_next_states = current_game_state.get_children_states();
        if possible_next_states.len() == 0 {
            return GameResult {
                player_a_wins: !player_a_turn,
                num_turns,
            };
        }

        num_turns += 1;
        let chosen_move_index = if player_a_turn {
            player_a_strategy.choose_move(true, &current_game_state, &possible_next_states)
        } else {
            player_b_strategy.choose_move(false, &current_game_state, &possible_next_states)
        };

        current_game_state = possible_next_states[chosen_move_index];
        player_a_turn = !player_a_turn;
    }
}

pub struct SimulationGameResult {
    pub player_a_wins: bool,
    pub num_turns: usize,
    pub player_a_is_strategy_1: bool,
}

pub struct SimulationResult {
    pub strategy_1_wins: usize,
    pub strategy_2_wins: usize,
    pub raw_games: Vec<SimulationGameResult>,
}

pub async fn simulate_random_games<
    S1: Strategy<GameState=GS> + 'static,
    S2: Strategy<GameState=GS> + 'static,
    GS: GameState + 'static
>(strategy_1: &S1, strategy_2: &S2, num_games: usize, initial_block_count: usize) -> SimulationResult {
    let mut tasks = Vec::with_capacity(num_games);

    let mut rng = rand::thread_rng();
    let random_states = (0..num_games).map(|_| {
        GS::from_generic_game_state(&GenericGameState::generate_random_state_with_blocks_rng(&mut rng, initial_block_count))
    }).collect::<Vec<_>>();

    for (i, initial_state) in random_states.into_iter().enumerate() {
        let player_a_is_strategy_1 = i % 2 == 0;

        let strategy_1 = strategy_1.clone();
        let strategy_2 = strategy_2.clone();

        let task = tokio::spawn(async move {
            let result = if player_a_is_strategy_1 {
                play_game(&strategy_1, &strategy_2, initial_state)
            } else {
                play_game(&strategy_2, &strategy_1, initial_state)
            };
            return SimulationGameResult {
                player_a_wins: result.player_a_wins,
                num_turns: result.num_turns,
                player_a_is_strategy_1,
            };
        });

        tasks.push(task);
    }

    let mut strategy_1_wins = 0;
    let mut strategy_2_wins = 0;
    let mut raw_games = Vec::with_capacity(num_games);
    for task in tasks {
        let result = task.await.unwrap();
        if result.player_a_wins {
            if result.player_a_is_strategy_1 {
                strategy_1_wins += 1;
            } else {
                strategy_2_wins += 1;
            }
        } else {
            if result.player_a_is_strategy_1 {
                strategy_2_wins += 1;
            } else {
                strategy_1_wins += 1;
            }
        }
        raw_games.push(result);
    }

    return SimulationResult {
        strategy_1_wins,
        strategy_2_wins,
        raw_games,
    };
}