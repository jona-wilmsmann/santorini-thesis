use std::env;
use std::sync::{Arc, Mutex};
use crate::game_state::{ContinuousBlockId, GameState, SimplifiedState};
use anyhow::Result;
use chrono::Local;
use num_format::ToFormattedString;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use crate::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use crate::precompute_state_winner::bit_vector::BitVector;
use crate::precompute_state_winner::bit_writer::BitWriter;

mod bit_vector;
mod bit_writer;
mod asset_valid_bit_count;

const CHUNK_SIZE_BYTES: u64 = 1024;

#[repr(u8)]
enum PresolveResult {
    PlayerAWinning = 0,
    PlayerBWinning = 1,
    Draw = 2,
}


fn get_chunk_amount(total_count: u64, bits_per_entry: u64, task_count: u64, task_id: u64) -> u64 {
    let total_chunk_amount = (total_count * bits_per_entry + (CHUNK_SIZE_BYTES * 8 - 1)) / (CHUNK_SIZE_BYTES * 8);

    let chunks_per_task = total_chunk_amount / task_count;
    let remainder = total_chunk_amount % task_count;

    return if task_id < remainder {
        chunks_per_task + 1
    } else {
        chunks_per_task
    };
}

async fn combine_partial_files(file_paths: Vec<String>, output_file_path: &str) -> Result<()> {
    let output_file = File::create(output_file_path).await?;
    let mut writer = BufWriter::new(output_file);

    let mut files = Vec::with_capacity(file_paths.len());
    for file_path in &file_paths {
        let file = File::open(file_path.clone()).await?;
        files.push(file);
    }

    let mut file_index = 0;
    loop {
        let mut buffer = [0; CHUNK_SIZE_BYTES as usize];
        let bytes_read = files[file_index].read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read]).await?;
        file_index = (file_index + 1) % files.len();
    }
    writer.flush().await?;

    for file_path in &file_paths {
        tokio::fs::remove_file(file_path).await?;
    }

    return Ok(());
}

fn presolve_state<
    GS: GameState + SimplifiedState + ContinuousBlockId,
    const BITS_PER_ENTRY: usize
>(state: &GS, parent_bit_vector: &Arc<BitVector<BITS_PER_ENTRY>>, reusable_child_states: &mut Vec<GS>) -> PresolveResult {
    let consider_draw: bool = BITS_PER_ENTRY == 2;

    state.get_children_states_reuse_vec(reusable_child_states);

    let player_a_turn = state.is_player_a_turn();

    if reusable_child_states.is_empty() {
        if consider_draw {
            return PresolveResult::Draw;
        }
        return if player_a_turn {
            PresolveResult::PlayerBWinning
        } else {
            PresolveResult::PlayerAWinning
        };
    }

    // If the active player has at least one child state that is winning, the active player wins
    // If the active player has at least one child state that is a draw, the game is a draw
    // If neither of the above conditions are met, the other player wins

    let mut can_force_draw = false;

    for child_state in reusable_child_states {
        if player_a_turn {
            if child_state.has_player_a_won() {
                return PresolveResult::PlayerAWinning;
            }
        } else {
            if child_state.has_player_b_won() {
                return PresolveResult::PlayerBWinning;
            }
        }
        let simplified_child_state = child_state.get_simplified_state();
        let child_continuous_block_id = simplified_child_state.get_continuous_block_id();

        let child_result = parent_bit_vector.get(child_continuous_block_id as usize);
        if player_a_turn {
            if child_result == PresolveResult::PlayerAWinning as u8 {
                return PresolveResult::PlayerAWinning;
            }
        } else {
            if child_result == PresolveResult::PlayerBWinning as u8 {
                return PresolveResult::PlayerBWinning;
            }
        }
        if consider_draw && !can_force_draw && child_result == PresolveResult::Draw as u8 {
            can_force_draw = true;
        }
    }

    if can_force_draw {
        return PresolveResult::Draw;
    }

    return if player_a_turn {
        PresolveResult::PlayerBWinning
    } else {
        PresolveResult::PlayerAWinning
    };
}

async fn update_solved_count(solved_count: &Arc<Mutex<u64>>, newly_solved: u64, total_count: u64, block_count: isize) {
    let mut solved_count = solved_count.lock().unwrap();

    let previous_percentage = *solved_count * 100 / total_count;
    *solved_count += newly_solved;
    let new_percentage = *solved_count * 100 / total_count;

    if new_percentage != previous_percentage {
        let local_time = Local::now();
        let formatted_time = local_time.format("%Y-%m-%d %H:%M:%S");
        println!("({}) Block {} - Progress: {}% of {} states", formatted_time, block_count, new_percentage, total_count.to_formatted_string(&num_format::Locale::en));
    }
}

/**
- BITS_PER_ENTRY = 1 => No draws are considered, if a player cannot move, the other player wins
- BITS_PER_ENTRY = 2 => Draws are considered, if a player cannot move, the game is a draw
 */
pub async fn presolve_state_winner<
    GS: GameState + SimplifiedState + ContinuousBlockId,
    const BITS_PER_ENTRY: usize,
>(block_count: isize, parallel_tasks: usize) -> Result<()> {
    assert!(BITS_PER_ENTRY == 1 || BITS_PER_ENTRY == 2);

    let data_folder_path = env::var("WINNER_DATA_FOLDER").expect("WINNER_DATA_FOLDER must be set");

    let continuous_block_id_count = GS::get_continuous_block_id_count(block_count);
    let parent_continuous_block_id_count = GS::get_continuous_block_id_count(block_count + 1);

    let parent_bit_vector = if parent_continuous_block_id_count != 0 {
        let bit_vector = BitVector::from_file_with_expected_length(
            &format!("{}/block{}_{}-{}.bin", data_folder_path, block_count + 1, 0, parent_continuous_block_id_count - 1),
            (parent_continuous_block_id_count as usize * BITS_PER_ENTRY + 7) / 8,
        ).await?;
        Arc::new(bit_vector)
    } else {
        Arc::new(BitVector::new_empty())
    };

    let mut tasks = Vec::new();
    let mut output_files = Vec::new();

    let global_solved_count = Arc::new(Mutex::new(0));

    for task_index in 0..parallel_tasks {
        let chunk_amount = get_chunk_amount(continuous_block_id_count, BITS_PER_ENTRY as u64, parallel_tasks as u64, task_index as u64);

        let output_file_path = format!("{}/block{}_part{}.bin", data_folder_path, block_count, task_index);
        output_files.push(output_file_path.clone());

        let parent_bit_vector = parent_bit_vector.clone();
        let global_solved_count = global_solved_count.clone();

        tasks.push(tokio::spawn(async move {
            let mut bit_writer = BitWriter::<BITS_PER_ENTRY>::new(output_file_path).await?;
            static UPDATE_INTERVAL: u64 = 100000;

            let mut reusable_child_states = Vec::new();
            let mut solved_count = 0;
            for chunk_index in 0..chunk_amount {
                let global_chunk_index = parallel_tasks as u64 * chunk_index + task_index as u64;
                let id_start = global_chunk_index * CHUNK_SIZE_BYTES * 8 / BITS_PER_ENTRY as u64;
                let id_end = ((global_chunk_index + 1) * CHUNK_SIZE_BYTES * 8 / BITS_PER_ENTRY as u64).min(continuous_block_id_count);

                for continuous_block_id in id_start..id_end {
                    let state = GS::from_continuous_block_id(block_count, continuous_block_id);
                    let winner = presolve_state::<GS, BITS_PER_ENTRY>(&state, &parent_bit_vector, &mut reusable_child_states);
                    bit_writer.write_data(winner as u8).await?;

                    solved_count += 1;
                    if solved_count % UPDATE_INTERVAL == 0 {
                        update_solved_count(&global_solved_count, UPDATE_INTERVAL, continuous_block_id_count, block_count).await;
                    }
                }
            }

            update_solved_count(&global_solved_count, solved_count % UPDATE_INTERVAL, continuous_block_id_count, block_count).await;

            bit_writer.flush().await?;
            return Ok::<(), anyhow::Error>(());
        }));
    }

    for task in tasks {
        task.await??;
    }

    println!("Presolved all states for block {}, combining files", block_count);

    if parallel_tasks != 1 {
        combine_partial_files(output_files, &format!("{}/block{}_{}-{}.bin", data_folder_path, block_count, 0, continuous_block_id_count - 1)).await?;
    }

    println!("Combined all files for block {}", block_count);

    return Ok(());
}


// Checks if a forced win for player B can be found in the current state with the given remaining moves
fn internal_find_forced_win(state: &GameState4x4Binary3Bit, max_moves: usize, player_a_turn: bool, bit_vectors: &Vec<BitVector<1>>, child_bit_vector_index: usize, reusable_child_states: &mut Vec<GameState4x4Binary3Bit>) -> bool {
    state.get_children_states_reuse_vec(reusable_child_states);

    if reusable_child_states.is_empty() {
        return player_a_turn;
    }

    if max_moves == 0 {
        return false;
    }

    let mut reusable_vec_for_child_states = Vec::with_capacity(32);

    let child_bit_vector = bit_vectors.get(child_bit_vector_index).unwrap();

    if player_a_turn {
        for child_state in reusable_child_states {
            if child_state.has_player_a_won() {
                return false;
            }
            let simplified_child_state = child_state.get_simplified_state();
            let child_block_id = simplified_child_state.get_continuous_block_id();
            if child_bit_vector.get(child_block_id as usize) == PresolveResult::PlayerAWinning as u8 {
                return false;
            }
            if !internal_find_forced_win(&simplified_child_state, max_moves - 1, false, bit_vectors, child_bit_vector_index + 1, &mut reusable_vec_for_child_states) {
                return false;
            }
        }
        return true;
    } else {
        for child_state in reusable_child_states {
            if child_state.has_player_b_won() {
                return true;
            }
            let simplified_child_state = child_state.get_simplified_state();
            let child_block_id = simplified_child_state.get_continuous_block_id();
            if child_bit_vector.get(child_block_id as usize) == PresolveResult::PlayerAWinning as u8 {
                continue;
            }
            if internal_find_forced_win(&simplified_child_state, max_moves - 1, true, bit_vectors, child_bit_vector_index + 1, &mut reusable_vec_for_child_states) {
                return true;
            }
        }
        return false;
    }
}


// This function is specific to the 4x4 binary 3-bit game state
// It finds the shorted forced win for player B
pub async fn find_shortest_forced_win() -> Result<usize> {
    type GS = GameState4x4Binary3Bit;

    let data_folder_path = env::var("WINNER_DATA_FOLDER").expect("WINNER_DATA_FOLDER must be set");

    let mut bit_vectors = Vec::new();

    let initial_block_id = -2;

    let initial_block_id_count = GS::get_continuous_block_id_count(initial_block_id);
    let expected_length = (initial_block_id_count as usize + 7) / 8;
    let bit_vector = BitVector::from_file_with_expected_length(&format!("{}/block{}_{}-{}.bin", data_folder_path, initial_block_id, 0, initial_block_id_count - 1), expected_length).await?;
    bit_vectors.push(bit_vector);


    let initial_state = GS::from_continuous_block_id(initial_block_id, 0);

    for move_limit in (1..=30).map(|x| x * 2) {
        println!("Checking move limit {}", move_limit);
        for block_id in (initial_block_id + move_limit as isize - 1)..=(initial_block_id + move_limit as isize) {
            let block_id_count = GS::get_continuous_block_id_count(block_id);
            let expected_length = (block_id_count as usize + 7) / 8;
            let bit_vector = BitVector::from_file_with_expected_length(&format!("{}/block{}_{}-{}.bin", data_folder_path, block_id, 0, block_id_count - 1), expected_length).await?;
            bit_vectors.push(bit_vector);
        }

        if internal_find_forced_win(&initial_state, move_limit, true, &bit_vectors, 1, &mut Vec::with_capacity(32)) {
            println!("Found forced win for move limit {}", move_limit);
            return Ok(move_limit);
        }
        println!("No forced win found for move limit {}", move_limit);
    }

    return Err(anyhow::anyhow!("No forced win found"));
}