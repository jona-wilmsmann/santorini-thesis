use std::env;
use serde::{Deserialize, Serialize};
use plotters::prelude::full_palette::{DEEPPURPLE, GREEN_900};
use crate::game_state::{GameState, SantoriniEval};
use crate::minimax::cached_minimax;
use crate::stats::benchmark_minimax_sorted::BenchmarkMinimaxSorted;
use crate::stats::StatGenerator;
use crate::stats::utils::draw_minimax_benchmark::{AverageMinimaxMeasurement, draw_minimax_benchmark, MinimaxBenchmarkData, MinimaxMeasurement};
use crate::stats::utils::gather_minimax_benchmark::gather_minimax_benchmark;

#[derive(Clone)]
pub struct BenchmarkMinimaxCached<GS: GameState + SantoriniEval + 'static, const SORTED_MIN_DEPTH_TO_SORT: usize, const MIN_DEPTH_TO_SORT: usize, const MIN_DEPTH_TO_CACHE: usize> {
    game_name: String,
    game_state_name: String,
    game_state_short_name: String,
    max_depth_cached: usize,
    number_cached_states: usize,
    block_count: usize,
    sorted_benchmark: BenchmarkMinimaxSorted<GS, SORTED_MIN_DEPTH_TO_SORT>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchmarkMinimaxCachedData {
    pub cpu_name: String,
    pub raw_measurements_cached: Vec<Vec<MinimaxMeasurement>>,
    pub average_measurements_cached: Vec<AverageMinimaxMeasurement>,
}



impl<
    GS: GameState + SantoriniEval + 'static,
    const SORTED_MIN_DEPTH_TO_SORT: usize,
    const MIN_DEPTH_TO_SORT: usize,
    const MIN_DEPTH_TO_CACHE: usize
> BenchmarkMinimaxCached<GS, SORTED_MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_CACHE> {
    pub fn new (
        game_name: String,
        game_state_name: String,
        game_state_short_name: String,
        max_depth_cached: usize,
        number_cached_states: usize,
        block_count: usize,
        sorted_benchmark: BenchmarkMinimaxSorted<GS, SORTED_MIN_DEPTH_TO_SORT>
    ) -> Self {
        assert_eq!(sorted_benchmark.number_sorted_states, number_cached_states);
        assert_eq!(sorted_benchmark.block_count, block_count);
        return BenchmarkMinimaxCached {
            game_name,
            game_state_name,
            game_state_short_name,
            max_depth_cached,
            number_cached_states,
            block_count,
            sorted_benchmark,
        };
    }
}


impl<
    GS: GameState + SantoriniEval + 'static,
    const SORTED_MIN_DEPTH_TO_SORT: usize,
    const MIN_DEPTH_TO_SORT: usize,
    const MIN_DEPTH_TO_CACHE: usize
> StatGenerator for BenchmarkMinimaxCached<GS, SORTED_MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_CACHE> {
    type DataType = BenchmarkMinimaxCachedData;

    fn get_stat_name(&self) -> String {
        return format!("minimax_cached_min{}x{}_{}", MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_CACHE, self.game_state_short_name);
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        let measurements = gather_minimax_benchmark(
            self.number_cached_states,
            self.block_count,
            self.max_depth_cached,
            cached_minimax::<GS, MIN_DEPTH_TO_SORT, MIN_DEPTH_TO_CACHE>
        ).await?;

        let cpu_name = env::var("CPU_NAME").unwrap_or("Unknown".to_string());
        return Ok(BenchmarkMinimaxCachedData {
            cpu_name,
            raw_measurements_cached: measurements.0,
            average_measurements_cached: measurements.1,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);

        let sorted_data_file = self.sorted_benchmark.get_most_recent_data_file()?;
        let sorted_data = self.sorted_benchmark.get_data(&sorted_data_file)?;
        assert_eq!(sorted_data.cpu_name, data.cpu_name);

        let sorted_name = if SORTED_MIN_DEPTH_TO_SORT == 0 {
            "Sorted Alpha-Beta Minimax".to_string()
        } else {
            format!("Sorted (d > {}) Alpha-Beta Minimax", SORTED_MIN_DEPTH_TO_SORT - 1)
        };

        let sorted_data = MinimaxBenchmarkData {
            label: sorted_name,
            cpu_name: sorted_data.cpu_name,
            color: GREEN_900,
            draw_execution_time_text: false,
            draw_game_states_text: false,
            average_measurements: sorted_data.average_measurements_sorted,
            raw_measurements: sorted_data.raw_measurements_sorted,
        };

        let cached_data = MinimaxBenchmarkData {
            label: "Cached Alpha-Beta Minimax".to_string(),
            cpu_name: data.cpu_name,
            color: DEEPPURPLE,
            draw_execution_time_text: true,
            draw_game_states_text: true,
            average_measurements: data.average_measurements_cached,
            raw_measurements: data.raw_measurements_cached,
        };

        return draw_minimax_benchmark(
            graph_path,
            format!("Cached Alpha-Beta Minimax - {} Benchmark", self.game_name),
            self.game_state_name.clone(),
            self.block_count,
            vec![sorted_data, cached_data]
        );
    }
}