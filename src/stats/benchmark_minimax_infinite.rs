use std::env;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::{Duration, Instant};
use fnv::FnvHashMap;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::game_state::{GameState, SantoriniEval};
use crate::generic_game_state::GenericGameState;
use crate::minimax::infinite_depth_minimax;
use crate::stats::StatGenerator;
use crate::stats::utils::formatters::ns_formatter;

#[derive(Clone)]
pub struct BenchmarkMinimaxInfinite<GS: GameState + SantoriniEval> {
    game_state_name: String,
    game_state_short_name: String,
    number_of_game_states: usize,
    block_counts: RangeInclusive<usize>,
    _phantom: std::marker::PhantomData<GS>,
}

#[derive(Serialize, Deserialize)]
pub struct InfiniteMinimaxMeasurement {
    pub block_count: usize,
    pub execution_time: Duration,
    pub cache_capacity: usize,
    pub player_a_wins: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AverageInfiniteMinimaxMeasurement {
    pub block_count: usize,
    pub average_execution_time: Duration,
    pub average_cache_capacity: usize,
    pub player_a_win_rate: f64,
}

#[derive(Serialize, Deserialize)]
pub struct BenchmarkMinimaxInfiniteData {
    pub cpu_name: String,
    pub raw_measurements: Vec<InfiniteMinimaxMeasurement>,
    pub average_measurements: Vec<AverageInfiniteMinimaxMeasurement>,
}

impl<GS: GameState + SantoriniEval> BenchmarkMinimaxInfinite<GS> {
    pub fn new(game_state_name: String, game_state_short_name: String, number_game_states: usize, block_counts: RangeInclusive<usize>) -> Self {
        return BenchmarkMinimaxInfinite {
            game_state_name,
            game_state_short_name,
            number_of_game_states: number_game_states,
            block_counts,
            _phantom: std::marker::PhantomData,
        };
    }
}


impl <GS: GameState + SantoriniEval + 'static> StatGenerator for BenchmarkMinimaxInfinite<GS> {
    type DataType = BenchmarkMinimaxInfiniteData;

    fn get_stat_name(&self) -> String {
        return format!("minimax_infinite_{}_b{}-{}", self.game_state_short_name, self.block_counts.start(), self.block_counts.end());
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        if cfg!(debug_assertions) {
            return Err(anyhow::anyhow!("Benchmarking should be done in release mode"));
        }


        let mut raw_measurements = Vec::with_capacity(self.number_of_game_states);

        for block_count in self.block_counts.clone().rev() {
            let mut rng = rand::rngs::StdRng::seed_from_u64(0);

            let random_states: Vec<GS> = (0..self.number_of_game_states)
                .map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_with_blocks_rng(&mut rng, block_count))).collect();

            let states_progress = Arc::new(Mutex::new(0));

            for state in random_states.into_iter() {
                let states_progress = states_progress.clone();
                let total_states = self.number_of_game_states;

                let mut cache = FnvHashMap::default();
                let start = Instant::now();
                let player_a_wins = infinite_depth_minimax(state, &mut cache);
                let execution_time = start.elapsed();
                let cache_capacity = cache.capacity();

                let mut states_progress_lock = states_progress.lock().await;
                *states_progress_lock += 1;
                println!("Block Count {} - Progress: {}/{}", block_count, *states_progress_lock, total_states);
                drop(states_progress_lock);

                let measurement = InfiniteMinimaxMeasurement {
                    block_count,
                    execution_time,
                    cache_capacity,
                    player_a_wins,
                };
                raw_measurements.push(measurement);
            }
        }

        let mut average_measurements = Vec::with_capacity(self.block_counts.clone().count());
        for block_count in self.block_counts.clone() {
            let total_execution_time = raw_measurements.iter().filter(|m| m.block_count == block_count).map(|m| m.execution_time).sum::<Duration>();
            let average_execution_time = total_execution_time / self.number_of_game_states as u32;

            let total_cache_capacity = raw_measurements.iter().filter(|m| m.block_count == block_count).map(|m| m.cache_capacity).sum::<usize>();
            let average_cache_capacity = total_cache_capacity / self.number_of_game_states;

            let player_a_wins = raw_measurements.iter().filter(|m| m.block_count == block_count).filter(|m| m.player_a_wins).count();
            let player_a_win_rate = player_a_wins as f64 / self.number_of_game_states as f64;

            average_measurements.push(AverageInfiniteMinimaxMeasurement {
                block_count,
                average_execution_time,
                average_cache_capacity,
                player_a_win_rate,
            });
        }

        let cpu_name = env::var("CPU_NAME").unwrap_or("Unknown".to_string());
        return Ok(BenchmarkMinimaxInfiniteData {
            cpu_name,
            raw_measurements,
            average_measurements,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let width = 1000;
        let height = 500;

        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);

        let root = SVGBackend::new(&graph_path, (width, height)).into_drawing_area();
        root.fill(&WHITE)?;

        let graph_upper_bound = 1e12 as usize;
        let log_duration_range = (0..graph_upper_bound).log_scale();

        // Line chart showing computation time, bar chart showing evaluated states
        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 70)
            .set_label_area_size(LabelAreaPosition::Bottom, 50)
            .build_cartesian_2d(*self.block_counts.start() .. *self.block_counts.end(), log_duration_range)?;

        chart
            .configure_mesh()
            .y_labels(13)
            .y_desc("Execution Time")
            .x_desc("Block Count")
            .y_label_formatter(&|y| ns_formatter(y))
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .draw()?;


        // draw line series
        chart.draw_series(LineSeries::new(
            data.average_measurements.iter().map(|m| (m.block_count, m.average_execution_time.as_nanos() as usize)),
            &BLUE,
        ))?;


        // add footnote in bottom right corner
        let game_state_footnote_text = Text::new(
            format!("N = {} game states per block count", self.number_of_game_states),
            (width as i32 - 10, height as i32 - 10),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Right, VPos::Bottom)),
        );
        root.draw(&game_state_footnote_text)?;
        let encoding_footnote_text = Text::new(
            format!("Encoding: {}", self.game_state_name),
            (width as i32 - 10, height as i32 - 23),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Right, VPos::Bottom)),
        );
        root.draw(&encoding_footnote_text)?;

        // add footnote in bottom left corner

        let cpu_footnote_text = Text::new(
            format!("CPU: {}", data.cpu_name),
            (10, height as i32 - 23),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Left, VPos::Bottom)),
        );
        root.draw(&cpu_footnote_text)?;


        root.present()?;

        return Ok(());
    }
}