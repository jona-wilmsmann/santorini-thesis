use std::ops::{RangeInclusive};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use anyhow::Result;
use crate::game_state::{GameState, SimplifiedState, MinimaxReady};
use crate::measurements::parallelize_measurements::parallelize_measurements;
use crate::minimax::readable_minmax_value;

pub async fn create_csv_report<
    GS: GameState + MinimaxReady + SimplifiedState + 'static
>(random_state_amount: usize, block_amounts: RangeInclusive<usize>, depths: RangeInclusive<usize>) -> Result<()> {
    let measurements = parallelize_measurements::<GS>(random_state_amount, block_amounts.clone(), depths.clone()).await;

    let folder_path = Path::new("reports");
    if !folder_path.exists() {
        fs::create_dir(folder_path).await?;
    }

    // File path uses current timestamp for uniqueness
    let file_path = folder_path.join(format!("measurements_{}_blocks{}-{}_depths{}-{}.csv", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(), block_amounts.start(), block_amounts.end(), depths.start(), depths.end()));

    let mut wtr = csv::Writer::from_path(file_path)?;
    wtr.write_record(&["Game State", "Block Amount", "Depth", "Result", "Calculation time", "Evaluated states", "Pruned states"])?;
    for measurement in &measurements {
        wtr.write_record(&[
            format!("{}", measurement.game_state.raw_value()),
            format!("{}", measurement.game_state_block_amount),
            format!("{}", measurement.depth),
            format!("{}", readable_minmax_value(measurement.result)),
            format!("{}", measurement.calculation_time.as_secs_f64()),
            format!("{}", measurement.evaluated_states),
            format!("{}", measurement.pruned_states),
        ])?;
    }
    wtr.flush()?;

    return Ok(());
}