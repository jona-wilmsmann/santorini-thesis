use std::env;
use plotters::prelude::*;
use plotters::prelude::full_palette::{BLUE_500, RED_400};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::game_state::ContinuousBlockId;
use crate::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use crate::stats::StatGenerator;

pub struct PresolveAnalysis {}

#[derive(Serialize, Deserialize)]
pub struct PresolveBlockData {
    pub block_count: usize,
    pub player_a_winning_states: u64,
    pub player_b_winning_states: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PresolveAnalysisData {
    pub block_data: Vec<PresolveBlockData>,
}


impl PresolveAnalysis {
    pub fn new() -> PresolveAnalysis {
        return PresolveAnalysis {};
    }
}

impl StatGenerator for PresolveAnalysis {
    type DataType = PresolveAnalysisData;

    fn get_stat_name(&self) -> String {
        return "presolve_analysis_4x4".to_string();
    }

    async fn gather_data(&self) -> anyhow::Result<PresolveAnalysisData> {
        let mut block_data = Vec::new();

        let winner_data_folder_path = env::var("WINNER_DATA_FOLDER").expect("WINNER_DATA_FOLDER must be set");
        for block_count in 0..=60 {
            let block_id_count = GameState4x4Binary3Bit::get_continuous_block_id_count(block_count) as usize;
            let presolve_file = format!("{}/block{}_{}-{}.bin", winner_data_folder_path, block_count, 0, block_id_count - 1);
            let mut file = File::open(presolve_file).await.unwrap();

            let expected_size = (block_id_count + 7) / 8;
            let mut buffer = Vec::with_capacity(expected_size);
            file.read_to_end(&mut buffer).await.unwrap();

            let mut zero_count = 0;
            let mut one_count = 0;

            // Read the full file, counting zeroes and ones
            // Read into u64, then use count_ones and count_zeros
            let mut u64_index = 0;
            while u64_index < buffer.len() / 8 {
                let value = u64::from_ne_bytes(buffer[u64_index * 8..u64_index * 8 + 8].try_into().unwrap());
                zero_count += value.count_zeros() as u64;
                one_count += value.count_ones() as u64;
                u64_index += 1;
            }

            // Handle the remaining bytes
            let remaining_bytes = buffer.len() % 8;
            if remaining_bytes > 0 {
                let mut value = 0u64;
                for i in 0..remaining_bytes {
                    value |= (buffer[u64_index * 8 + i] as u64) << (i * 8);
                }
                zero_count += value.count_zeros() as u64;
                one_count += value.count_ones() as u64;
                u64_index += 1;
            }

            zero_count -= (u64_index * 64 - block_id_count) as u64;

            block_data.push(PresolveBlockData {
                block_count,
                player_a_winning_states: zero_count,
                player_b_winning_states: one_count,
            });

            println!("Block count: {}, zeroes: {}, ones: {}", block_count, zero_count, one_count);
            println!("Player A win ratio: {}", zero_count as f64 / block_id_count as f64);
        }

        return Ok(PresolveAnalysisData {
            block_data
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        // Log scale

        let log_graph_path = format!("{}/{}.svg", output_folder_path, data_time);
        let root = SVGBackend::new(&log_graph_path, (1024, 360)).into_drawing_area();

        root.fill(&WHITE)?;

        let mut chart_log = ChartBuilder::on(&root)
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d((0usize..60).into_segmented(), 0.0..1.0)?;


        chart_log
            .configure_mesh()
            .disable_x_mesh()
            .y_desc("% Game States")
            .x_desc("Block Count")
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .draw()?;

        chart_log.draw_series(
            Histogram::vertical(&chart_log)
                .style(RED_400.filled())
                .margin(1)
                .data(data.block_data.iter().map(|block_data| (block_data.block_count, 1.0)))
        )?
            .label("Player B winning states")
            .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], RED_400.filled()));

        chart_log.draw_series(
            Histogram::vertical(&chart_log)
                .style(BLUE_500.filled())
                .margin(1)
                .data(data.block_data.iter().map(|block_data| (block_data.block_count, (block_data.player_a_winning_states as f64) / (block_data.player_a_winning_states + block_data.player_b_winning_states) as f64)))
        )?
            .label("Player A winning states")
            .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], BLUE_500.filled()));

        chart_log.configure_series_labels()
            .position(SeriesLabelPosition::UpperMiddle)
            .background_style(&WHITE)
            .border_style(&BLACK)
            .draw()?;

        root.present()?;

        return Ok(());
    }
}