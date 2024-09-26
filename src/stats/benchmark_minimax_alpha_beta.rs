use std::env;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use plotters::prelude::*;
use rand::SeedableRng;
use crate::game_state::GameState;
use crate::generic_game_state::GenericGameState;
use crate::minimax::{alpha_beta_minimax, simple_minimax};
use crate::stats::benchmark_minimax_simple::BenchmarkMinimaxSimple;
use crate::stats::StatGenerator;
use crate::stats::utils::draw_minimax_benchmark::{AverageMinimaxMeasurement, draw_minimax_benchmark, MinimaxBenchmarkData, MinimaxMeasurement};
use crate::stats::utils::gather_minimax_benchmark::gather_minimax_benchmark;

pub struct BenchmarkMinimaxAlphaBeta<GS: GameState> {
    game_name: String,
    game_state_name: String,
    game_state_short_name: String,
    pub(crate) max_depth_alpha_beta: usize,
    pub(crate) number_alpha_beta_game_states: usize,
    pub(crate) block_count: usize,
    simple_benchmark: BenchmarkMinimaxSimple<GS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchmarkMinimaxAlphaBetaData {
    cpu_name: String,
    raw_measurements_simple: Vec<Vec<MinimaxMeasurement>>,
    average_measurements_simple: Vec<AverageMinimaxMeasurement>,
    pub(crate) raw_measurements_alpha_beta: Vec<Vec<MinimaxMeasurement>>,
    pub(crate) average_measurements_alpha_beta: Vec<AverageMinimaxMeasurement>,
}


impl<GS: GameState> BenchmarkMinimaxAlphaBeta<GS> {
    pub fn new(game_name: String, game_state_name: String, game_state_short_name: String, max_depth_alpha_beta: usize, number_alpha_beta_game_states: usize, block_count: usize, simple_benchmark: BenchmarkMinimaxSimple<GS>) -> Self {
        assert_eq!(simple_benchmark.number_simple_game_states, number_alpha_beta_game_states);
        assert_eq!(simple_benchmark.block_count, block_count);
        return BenchmarkMinimaxAlphaBeta {
            game_name,
            game_state_name,
            game_state_short_name,
            max_depth_alpha_beta,
            number_alpha_beta_game_states,
            block_count,
            simple_benchmark,
        };
    }
}


impl<GS: GameState + 'static> StatGenerator for BenchmarkMinimaxAlphaBeta<GS> {
    type DataType = BenchmarkMinimaxAlphaBetaData;

    fn get_stat_name(&self) -> String {
        return format!("benchmark_minimax_alpha_beta_{}", self.game_state_short_name);
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        let measurements = gather_minimax_benchmark(
            self.number_alpha_beta_game_states,
            self.block_count,
            self.max_depth_alpha_beta,
            alpha_beta_minimax::<GS>
        ).await?;

        let simple_data_file = self.simple_benchmark.get_most_recent_data_file()?;
        let simple_data = self.simple_benchmark.get_data(&simple_data_file)?;

        let cpu_name = env::var("CPU_NAME").unwrap_or("Unknown".to_string());
        return Ok(BenchmarkMinimaxAlphaBetaData {
            cpu_name,
            raw_measurements_simple: simple_data.raw_measurements_simple,
            average_measurements_simple: simple_data.average_measurements_simple,
            raw_measurements_alpha_beta: measurements.0,
            average_measurements_alpha_beta: measurements.1,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);

        let simple_data = MinimaxBenchmarkData {
            label: "Simple Minimax".to_string(),
            cpu_name: data.cpu_name.clone(),
            color: BLUE,
            draw_execution_time_text: true,
            draw_game_states_text: true,
            average_measurements: data.average_measurements_simple,
            raw_measurements: data.raw_measurements_simple,
        };

        let alpha_beta_data = MinimaxBenchmarkData {
            label: "Alpha-Beta Minimax".to_string(),
            cpu_name: data.cpu_name,
            color: RED,
            draw_execution_time_text: true,
            draw_game_states_text: true,
            average_measurements: data.average_measurements_alpha_beta,
            raw_measurements: data.raw_measurements_alpha_beta,
        };

        return draw_minimax_benchmark(graph_path, self.game_name.clone(), self.game_state_name.clone(), vec![simple_data, alpha_beta_data]);
    }
}