use std::sync::{Arc, Mutex};
use crate::game_state::GameState;
use anyhow::Result;
use chrono::Local;
use num_format::ToFormattedString;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use crate::precompute_state_winner::bit_vector::BitVector;
use crate::precompute_state_winner::bit_writer::BitWriter;

mod bit_vector;
mod bit_writer;

const CHUNK_SIZE_BYTES: u64 = 1024;

fn get_chunk_amount(total_count: u64, task_count: u64, task_id: u64) -> u64 {
    let total_chunk_amount = (total_count + (CHUNK_SIZE_BYTES * 8 - 1)) / (CHUNK_SIZE_BYTES * 8);

    let chunks_per_task = total_chunk_amount / task_count;
    let remainder = total_chunk_amount % task_count;

    return if task_id < remainder {
        chunks_per_task + 1
    } else {
        chunks_per_task
    }
}

async fn combine_partial_files(file_paths: Vec<String>, output_file_path: &str) -> Result<()> {
    let output_file = File::create(output_file_path).await?;
    let mut writer = BufWriter::new(output_file);

    let mut files = Vec::new();
    for file_path in &file_paths {
        let file = File::open(file_path.clone()).await?;
        files.push(file);
    }

    let mut file_index = 0;
    loop {
        let mut buffer = [0; CHUNK_SIZE_BYTES as usize];
        let mut bytes_read = files[file_index].read(&mut buffer).await?;
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

fn presolve_state(state: &GameState, parent_bit_vector: &Arc<BitVector>) -> bool {
    let child_states = state.get_children_states();

    if child_states.is_empty() {
        return false;
    }

    // If any of the child states are losing for the then active player, then the current state is winning
    for child_state in child_states {
        if child_state.has_player_a_won() {
            return true;
        }
        let flipped_child_state = child_state.get_flipped_state();
        let simplified_flipped_child_state = flipped_child_state.get_symmetric_simplified_state();
        let child_continuous_block_id = simplified_flipped_child_state.get_continuous_block_id();
        if !parent_bit_vector.get(child_continuous_block_id as usize) {
            return true;
        }
    }

    return false;
}

async fn update_solved_count(solved_count: &Arc<Mutex<u64>>, newly_solved: u64, total_count: u64, block_count: usize) {
    let mut solved_count = solved_count.lock().unwrap();

    let previous_percentage = *solved_count * 100 / total_count;
    *solved_count += newly_solved;
    let new_percentage = *solved_count * 100 / total_count;

    if new_percentage != previous_percentage{
        let local_time = Local::now();
        let formatted_time = local_time.format("%Y-%m-%d %H:%M:%S");
        println!("({}) Block {} - Progress: {}% of {} states", formatted_time, block_count, new_percentage, total_count.to_formatted_string(&num_format::Locale::en));
    }
}

pub async fn presolve_state_winner(block_count: usize, parallel_tasks: usize, data_folder_path: &str) -> Result<()> {
    let continuous_block_id_count = GameState::get_continuous_block_id_count(block_count);
    let parent_continuous_block_id_count = GameState::get_continuous_block_id_count(block_count + 1);

    let parent_bit_vector = if parent_continuous_block_id_count != 0 {
        let bit_vector = BitVector::from_file(&format!("{}/block{}_{}-{}.bin", data_folder_path, block_count + 1, 0, parent_continuous_block_id_count - 1)).await?;
        Arc::new(bit_vector)
    } else {
        Arc::new(BitVector::new_empty())
    };

    let mut tasks = Vec::new();
    let mut output_files = Vec::new();

    let global_solved_count = Arc::new(Mutex::new(0));

    for task_index in 0..parallel_tasks {
        let chunk_amount = get_chunk_amount(continuous_block_id_count, parallel_tasks as u64, task_index as u64);

        let output_file_path = format!("{}/block{}_part{}.bin", data_folder_path, block_count, task_index);
        output_files.push(output_file_path.clone());

        let parent_bit_vector = parent_bit_vector.clone();
        let global_solved_count = global_solved_count.clone();

        tasks.push(tokio::spawn(async move {
            let mut bit_writer = BitWriter::new(output_file_path).await?;
            static UPDATE_INTERVAL: u64 = 100000;

            let mut solved_count = 0;
            for chunk_index in 0..chunk_amount {
                let global_chunk_index = parallel_tasks as u64 * chunk_index + task_index as u64;
                let id_start = global_chunk_index * CHUNK_SIZE_BYTES * 8;
                let id_end = ((global_chunk_index + 1) * CHUNK_SIZE_BYTES * 8).min(continuous_block_id_count);

                for continuous_block_id in id_start..id_end {
                    let state = GameState::from_continuous_block_id(block_count, continuous_block_id);
                    let winner = presolve_state(&state, &parent_bit_vector);
                    bit_writer.write_bit(winner).await?;

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