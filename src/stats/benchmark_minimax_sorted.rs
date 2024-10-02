use std::env;
use serde::{Deserialize, Serialize};
use plotters::prelude::*;
use plotters::prelude::full_palette::GREEN_900;
use crate::game_state::{GameState, MinimaxReady};
use crate::minimax::alpha_beta_sorted_minimax;
use crate::stats::benchmark_minimax_alpha_beta::BenchmarkMinimaxAlphaBeta;
use crate::stats::StatGenerator;
use crate::stats::utils::draw_minimax_benchmark::{AverageMinimaxMeasurement, draw_minimax_benchmark, MinimaxBenchmarkData, MinimaxMeasurement};
use crate::stats::utils::gather_minimax_benchmark::gather_minimax_benchmark;

#[derive(Clone)]
pub struct BenchmarkMinimaxSorted<GS: GameState + MinimaxReady + 'static, const MIN_DEPTH_TO_SORT: usize> {
    game_name: String,
    game_state_name: String,
    game_state_short_name: String,
    pub(crate) max_depth_sorted: usize,
    pub(crate) number_sorted_states: usize,
    pub(crate) block_count: usize,
    alpha_beta_benchmark: BenchmarkMinimaxAlphaBeta<GS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchmarkMinimaxSortedData {
    pub cpu_name: String,
    pub raw_measurements_sorted: Vec<Vec<MinimaxMeasurement>>,
    pub average_measurements_sorted: Vec<AverageMinimaxMeasurement>,
}



impl<GS: GameState + MinimaxReady + 'static, const MIN_DEPTH_TO_SORT: usize> BenchmarkMinimaxSorted<GS, MIN_DEPTH_TO_SORT> {
    pub fn new (
        game_name: String,
        game_state_name: String,
        game_state_short_name: String,
        max_depth_sorted: usize,
        number_sorted_states: usize,
        block_count: usize,
        alpha_beta_benchmark: BenchmarkMinimaxAlphaBeta<GS>
    ) -> Self {
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


impl<GS: GameState + MinimaxReady + 'static, const MIN_DEPTH_TO_SORT: usize> StatGenerator for BenchmarkMinimaxSorted<GS, MIN_DEPTH_TO_SORT> {
    type DataType = BenchmarkMinimaxSortedData;

    fn get_stat_name(&self) -> String {
        return format!("minimax_sorted_min{}_{}", MIN_DEPTH_TO_SORT, self.game_state_short_name);
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        let measurements = gather_minimax_benchmark(
            self.number_sorted_states,
            self.block_count,
            self.max_depth_sorted,
            alpha_beta_sorted_minimax::<GS, MIN_DEPTH_TO_SORT>
        ).await?;

        let cpu_name = env::var("CPU_NAME").unwrap_or("Unknown".to_string());
        return Ok(BenchmarkMinimaxSortedData {
            cpu_name,
            raw_measurements_sorted: measurements.0,
            average_measurements_sorted: measurements.1,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);

        let alpha_beta_data_file = self.alpha_beta_benchmark.get_most_recent_data_file()?;
        let alpha_beta_data = self.alpha_beta_benchmark.get_data(&alpha_beta_data_file)?;
        assert_eq!(alpha_beta_data.cpu_name, data.cpu_name);

        let sorted_name = if MIN_DEPTH_TO_SORT == 0 {
            "Sorted Alpha-Beta Minimax".to_string()
        } else {
            format!("Sorted (d > {}) Alpha-Beta Minimax", MIN_DEPTH_TO_SORT - 1)
        };

        let alpha_beta_data = MinimaxBenchmarkData {
            label: "Alpha-Beta Minimax".to_string(),
            cpu_name: alpha_beta_data.cpu_name,
            color: RED,
            draw_execution_time_text: false,
            draw_game_states_text: false,
            average_measurements: alpha_beta_data.average_measurements_alpha_beta,
            raw_measurements: alpha_beta_data.raw_measurements_alpha_beta,
        };

        let sorted_data = MinimaxBenchmarkData {
            label: sorted_name.clone(),
            cpu_name: data.cpu_name,
            color: GREEN_900,
            draw_execution_time_text: true,
            draw_game_states_text: true,
            average_measurements: data.average_measurements_sorted,
            raw_measurements: data.raw_measurements_sorted,
        };

        return draw_minimax_benchmark(
            graph_path,
            format!("{} - {} Benchmark", sorted_name, self.game_name),
            self.game_state_name.clone(),
            self.block_count,
            vec![alpha_beta_data, sorted_data]
        );
    }
}