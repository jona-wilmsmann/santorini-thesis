use std::cmp::max;
use std::env;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use plotters::prelude::*;
use plotters::prelude::full_palette::DEEPPURPLE;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use rand::SeedableRng;
use crate::game_state::{GameState, MinimaxReady};
use crate::generic_game_state::GenericGameState;
use crate::minimax::{alpha_beta_minimax, alpha_beta_sorted_minimax};
use crate::stats::benchmark_minimax_alpha_beta::BenchmarkMinimaxAlphaBeta;
use crate::stats::StatGenerator;
use crate::stats::utils::draw_minimax_benchmark::{AverageMinimaxMeasurement, draw_minimax_benchmark, MinimaxBenchmarkData, MinimaxMeasurement};
use crate::stats::utils::formatters::{ns_formatter, value_formatter};
use crate::stats::utils::gather_minimax_benchmark::gather_minimax_benchmark;

pub struct BenchmarkMinimaxSorted<GS: GameState + MinimaxReady> {
    game_name: String,
    game_state_name: String,
    game_state_short_name: String,
    max_depth_sorted: usize,
    number_sorted_states: usize,
    block_count: usize,
    alpha_beta_benchmark: BenchmarkMinimaxAlphaBeta<GS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchmarkGameStatesBasicData {
    cpu_name: String,
    raw_measurements_alpha_beta: Vec<Vec<MinimaxMeasurement>>,
    average_measurements_alpha_beta: Vec<AverageMinimaxMeasurement>,
    raw_measurements_sorted: Vec<Vec<MinimaxMeasurement>>,
    average_measurements_sorted: Vec<AverageMinimaxMeasurement>,
}



impl<GS: GameState + MinimaxReady> BenchmarkMinimaxSorted<GS> {
    pub fn new (game_name: String, game_state_name: String, game_state_short_name: String, max_depth_sorted: usize, number_sorted_states: usize, block_count: usize, alpha_beta_benchmark: BenchmarkMinimaxAlphaBeta<GS>) -> Self {
        assert_eq!(alpha_beta_benchmark.number_alpha_beta_game_states, number_sorted_states);
        assert_eq!(alpha_beta_benchmark.block_count, block_count);
        return BenchmarkMinimaxSorted {
            game_name,
            game_state_name,
            game_state_short_name,
            max_depth_sorted,
            number_sorted_states,
            block_count,
            alpha_beta_benchmark,
        };
    }
}


impl<GS: GameState + MinimaxReady + 'static> StatGenerator for BenchmarkMinimaxSorted<GS> {
    type DataType = BenchmarkGameStatesBasicData;

    fn get_stat_name(&self) -> String {
        return format!("benchmark_minimax_sorted_{}", self.game_state_short_name);
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        let measurements = gather_minimax_benchmark(
            self.number_sorted_states,
            self.block_count,
            self.max_depth_sorted,
            alpha_beta_sorted_minimax::<GS>
        ).await?;

        let alpha_beta_data_file = self.alpha_beta_benchmark.get_most_recent_data_file()?;
        let alpha_beta_data = self.alpha_beta_benchmark.get_data(&alpha_beta_data_file)?;

        let cpu_name = env::var("CPU_NAME").unwrap_or("Unknown".to_string());
        return Ok(BenchmarkGameStatesBasicData {
            cpu_name,
            raw_measurements_alpha_beta: alpha_beta_data.raw_measurements_alpha_beta,
            average_measurements_alpha_beta: alpha_beta_data.average_measurements_alpha_beta,
            raw_measurements_sorted: measurements.0,
            average_measurements_sorted: measurements.1,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);

        let alpha_beta_data = MinimaxBenchmarkData {
            label: "Alpha-Beta Minimax".to_string(),
            cpu_name: data.cpu_name.clone(),
            color: RED,
            draw_execution_time_text: true,
            draw_game_states_text: true,
            average_measurements: data.average_measurements_alpha_beta,
            raw_measurements: data.raw_measurements_alpha_beta,
        };

        let sorted_data = MinimaxBenchmarkData {
            label: "Sorted Alpha-Beta Minimax".to_string(),
            cpu_name: data.cpu_name,
            color: DEEPPURPLE,
            draw_execution_time_text: true,
            draw_game_states_text: true,
            average_measurements: data.average_measurements_sorted,
            raw_measurements: data.raw_measurements_sorted,
        };

        return draw_minimax_benchmark(graph_path, self.game_name.clone(), self.game_state_name.clone(), vec![alpha_beta_data, sorted_data]);
    }
}