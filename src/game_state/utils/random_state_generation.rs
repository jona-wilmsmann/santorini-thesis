use rand::Rng;
use crate::game_state::generic_game_state::GenericGameState;


pub fn generate_random_state_with_blocks(mut block_amount: usize) -> GenericGameState {
    let mut tile_heights = [0; 16];

    let mut rng = rand::thread_rng();

    while block_amount > 0 {
        loop {
            let tile = rng.gen_range(0..16) as usize;
            if tile_heights[tile] < 4 {
                tile_heights[tile] += 1;
                break;
            }
        }
        block_amount -= 1;
    }

    let player_a_tile;
    let player_b_tile;

    loop {
        let rand_tile = rng.gen_range(0..16) as u8;
        if tile_heights[rand_tile as usize] <= 2 {
            player_a_tile = rand_tile;
            break;
        }
    }
    loop {
        let rand_tile = rng.gen_range(0..16) as u8;
        if tile_heights[rand_tile as usize] <= 2 && rand_tile != player_a_tile {
            player_b_tile = rand_tile;
            break;
        }
    }

    return GenericGameState::new(player_a_tile, player_b_tile, tile_heights).expect("Randomly generated invalid game state");
}

pub fn _generate_random_state() -> GenericGameState {
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