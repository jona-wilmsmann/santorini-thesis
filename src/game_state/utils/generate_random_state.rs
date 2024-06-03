use rand::Rng;
use crate::game_state::generic_game_state::GenericGameState;

pub fn generate_random_state() -> GenericGameState {
    let mut tile_heights = [0; 16];
    let player_a_tile;
    let mut player_b_tile ;

    let mut rng = rand::thread_rng();

    player_a_tile = rng.gen_range(0..16) as u8;

    loop {
        player_b_tile = rng.gen_range(0..16) as u8;
        if player_b_tile != player_a_tile {
            break;
        }
    }

    for i in 0..16 {
        let max_height = if i == player_a_tile as usize || i == player_b_tile as usize { 2 } else { 4 };
        tile_heights[i] = rng.gen_range(0..max_height + 1);
    }

    return GenericGameState::new(player_a_tile, player_b_tile, tile_heights).expect("Randomly generated invalid game state");
}