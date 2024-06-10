use std::ops::Range;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use anyhow::Result;
use crate::minimax::readable_minmax_value;

pub async fn create_csv_report(random_state_amount: usize, block_amount: usize, depths: Range<usize>) -> Result<()> {
    let measurements = crate::measurements::parallelize_measurements::parallelize_measurements(random_state_amount, block_amount, depths).await;


    let folder_path = Path::new("reports");
    if !folder_path.exists() {
        fs::create_dir(folder_path).await?;
    }

    // File path uses current timestamp for uniqueness
    let file_path = folder_path.join(format!("measurements_{}_{}block.csv", SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(), block_amount));

    let mut wtr = csv::Writer::from_path(file_path)?;
    wtr.write_record(&["Game State", "Depth", "Result", "Calculation time", "Evaluated states", "Pruned states"])?;
    for measurement in &measurements {
        wtr.write_record(&[
            format!("{}", measurement.game_state.raw_value()),
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