use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use plotters::prelude::*;
use rand::SeedableRng;
use tokio::sync::Mutex;
use crate::game_state::GameState;
use crate::generic_game_state::GenericGameState;
use crate::minimax::simple_minimax;
use crate::stats::StatGenerator;
use crate::stats::utils::draw_minimax_benchmark::{AverageMinimaxMeasurement, draw_minimax_benchmark, MinimaxBenchmarkData, MinimaxMeasurement};
use crate::stats::utils::gather_minimax_benchmark::gather_minimax_benchmark;

pub struct BenchmarkMinimaxSimple<GS: GameState> {
    pub(crate) game_name: String,
    pub(crate) game_state_name: String,
    pub(crate) game_state_short_name: String,
    pub(crate) max_depth_simple: usize,
    pub(crate) number_simple_game_states: usize,
    pub(crate) block_count: usize,
    _phantom: std::marker::PhantomData<GS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchmarkMinimaxSimpleData {
    cpu_name: String,
    pub(crate) raw_measurements_simple: Vec<Vec<MinimaxMeasurement>>,
    pub(crate) average_measurements_simple: Vec<AverageMinimaxMeasurement>,
}


impl<GS: GameState> BenchmarkMinimaxSimple<GS> {
    pub fn new(game_name: String, game_state_name: String, game_state_short_name: String, max_depth_simple: usize, number_simple_game_states: usize, block_count: usize) -> Self {
        return BenchmarkMinimaxSimple {
            game_name,
            game_state_name,
            game_state_short_name,
            max_depth_simple,
            number_simple_game_states,
            block_count,
            _phantom: std::marker::PhantomData,
        };
    }
}


impl<GS: GameState + 'static> StatGenerator for BenchmarkMinimaxSimple<GS> {
    type DataType = BenchmarkMinimaxSimpleData;

    fn get_stat_name(&self) -> String {
        return format!("benchmark_minimax_simple_{}", self.game_state_short_name);
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        let measurements = gather_minimax_benchmark(
            self.number_simple_game_states,
            self.block_count,
            self.max_depth_simple,
            simple_minimax::<GS>
        ).await?;

        let cpu_name = env::var("CPU_NAME").unwrap_or("Unknown".to_string());
        return Ok(BenchmarkMinimaxSimpleData {
            cpu_name,
            raw_measurements_simple: measurements.0,
            average_measurements_simple: measurements.1,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let simple_data = MinimaxBenchmarkData {
            label: "Simple Minimax".to_string(),
            cpu_name: data.cpu_name,
            color: BLUE,
            draw_execution_time_text: true,
            draw_game_states_text: true,
            average_measurements: data.average_measurements_simple,
            raw_measurements: data.raw_measurements_simple,
        };
        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);

        return draw_minimax_benchmark(graph_path, "Simple Minimax Benchmark".to_string(), self.game_state_name.clone(), vec![simple_data]);
    }
}