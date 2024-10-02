use std::collections::HashMap;
use std::env;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::{Duration, Instant};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use rand::SeedableRng;
use tokio::sync::Mutex;
use crate::game_state::{GameState, MinimaxReady};
use crate::generic_game_state::GenericGameState;
use crate::minimax::minimax;
use crate::minimax::minimax_cache::MinimaxCache;
use crate::stats::StatGenerator;
use crate::stats::utils::formatters::{ns_formatter, value_formatter};

use plotters::prelude::*;
use plotters::prelude::full_palette::{BROWN, ORANGE, PURPLE};
use plotters::style::text_anchor::{HPos, Pos, VPos};

fn get_color(index: usize) -> RGBColor {
    match index {
        0 => RED,
        1 => BLUE,
        2 => GREEN,
        3 => MAGENTA,
        4 => CYAN,
        5 => BROWN,
        6 => PURPLE,
        7 => ORANGE,
        8 => BLACK,
        9 => YELLOW,
        _ => BLACK, // Fallback in case index exceeds 9
    }
}


#[derive(Clone)]
pub struct MinimaxSolveStats<GS: GameState + MinimaxReady> {
    game_state_name: String,
    game_state_short_name: String,
    depths: RangeInclusive<usize>,
    block_counts: RangeInclusive<usize>,
    number_game_states: usize,
    _phantom: std::marker::PhantomData<GS>,
}


#[derive(Serialize, Deserialize)]
pub struct SolveMeasurement {
    pub depth: usize,
    pub block_count: usize,
    pub result: f32,
    pub solved: bool,
    pub execution_time: Duration,
}

#[derive(Serialize, Deserialize)]
pub struct MinimaxSolveData {
    pub cpu_name: String,
    pub raw_measurements: Vec<SolveMeasurement>,
}


impl<GS: GameState + MinimaxReady> MinimaxSolveStats<GS> {
    pub fn new(game_state_name: String, game_state_short_name: String, depths: RangeInclusive<usize>, block_counts: RangeInclusive<usize>, number_game_states: usize) -> MinimaxSolveStats<GS> {
        return MinimaxSolveStats {
            game_state_name,
            game_state_short_name,
            depths,
            block_counts,
            number_game_states,
            _phantom: std::marker::PhantomData,
        };
    }
}


impl<GS: GameState + MinimaxReady + 'static> StatGenerator for MinimaxSolveStats<GS> {
    type DataType = MinimaxSolveData;

    fn get_stat_name(&self) -> String {
        return format!("solved_minimax_{}_d{}-{}_b{}-{}", self.game_state_short_name, self.depths.start(), self.depths.end(), self.block_counts.start(), self.block_counts.end());
    }

    async fn gather_data(&self) -> anyhow::Result<MinimaxSolveData> {
        if cfg!(debug_assertions) {
            return Err(anyhow::anyhow!("Benchmarking should be done in release mode"));
        }

        let mut raw_measurements = Vec::new();

        let mut rng = rand::rngs::StdRng::seed_from_u64(0);

        let mut tasks = Vec::new();

        for block_count in self.block_counts.clone().into_iter() {
            let block_state_progress = Arc::new(Mutex::new(0));

            let random_states: Vec<GS> = (0..self.number_game_states)
                .map(|_| GS::from_generic_game_state(&GenericGameState::generate_random_state_with_blocks_rng(&mut rng, block_count))).collect();

            for state in random_states.into_iter() {
                let block_state_progress = block_state_progress.clone();
                let total_states = self.number_game_states;
                let depths = self.depths.clone();

                tasks.push(tokio::spawn(async move {
                    let mut measurements = Vec::new();
                    for depth in depths {
                        let start = Instant::now();
                        let result = minimax(&state, depth, f32::NEG_INFINITY, f32::INFINITY, &mut MinimaxCache::new());
                        let computation_time = start.elapsed();
                        measurements.push(SolveMeasurement {
                            depth,
                            block_count,
                            result,
                            solved: result.is_infinite(),
                            execution_time: computation_time,
                        });
                    }
                    let mut block_state_progress_lock = block_state_progress.lock().await;
                    *block_state_progress_lock += 1;
                    println!("Block Count {} - Progress: {}/{}", block_count, *block_state_progress_lock, total_states);
                    drop(block_state_progress_lock);
                    return measurements;
                }));
            }
        }

        for task in tasks.into_iter() {
            let measurements = task.await?;
            raw_measurements.extend(measurements);
        }

        let cpu_name = env::var("CPU_NAME").unwrap_or("Unknown".to_string());

        return Ok(MinimaxSolveData {
            cpu_name,
            raw_measurements,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        struct AverageMeasurement {
            block_count: usize,
            depth: usize,
            solved: f64,
            player_a_wins: f64,
            player_b_wins: f64,
            execution_time_ns: usize,
        }

        let measurements_map: HashMap<(usize, usize), Vec<&SolveMeasurement>> = data.raw_measurements.iter().fold(HashMap::new(), |mut acc, m| {
            acc.entry((m.block_count, m.depth)).or_insert(Vec::new()).push(m);
            return acc;
        });

        let mut averaged_measurements = Vec::new();

        for block_count in self.block_counts.clone() {
            for depth in self.depths.clone() {
                let measurements = measurements_map.get(&(block_count, depth)).unwrap();
                let total_time_ns = measurements.iter().map(|m| m.execution_time.as_nanos() as usize).sum::<usize>();
                let total_solved = measurements.iter().filter(|m| m.solved).count();
                let total_player_a_wins = measurements.iter().filter(|m| m.result == f32::INFINITY).count();
                let total_player_b_wins = measurements.iter().filter(|m| m.result == f32::NEG_INFINITY).count();

                let average_time_ns = total_time_ns / self.number_game_states;
                let average_solved = total_solved as f64 / self.number_game_states as f64;
                let average_player_a_wins = total_player_a_wins as f64 / self.number_game_states as f64;
                let average_player_b_wins = total_player_b_wins as f64 / self.number_game_states as f64;

                if depth == 8 {
                    println!("Block Count: {}, Depth: {}, Solved: {}, Player A Wins: {}, Player B Wins: {}, Execution Time: {}", block_count, depth, average_solved, average_player_a_wins, average_player_b_wins, average_time_ns);
                }

                averaged_measurements.push(AverageMeasurement {
                    block_count,
                    depth,
                    solved: average_solved,
                    player_a_wins: average_player_a_wins,
                    player_b_wins: average_player_b_wins,
                    execution_time_ns: average_time_ns,
                });
            }
        }


        let solve_graph_path = format!("{}/{}_solve.svg", output_folder_path, data_time);
        let time_graph_path = format!("{}/{}_time.svg", output_folder_path, data_time);

        let width = 1000;
        let height = 500;

        let time_root = SVGBackend::new(&time_graph_path, (width, height)).into_drawing_area();
        time_root.fill(&WHITE)?;

        let log_duration_range = (0..1e10 as usize).log_scale();

        let mut time_chart = ChartBuilder::on(&time_root)
            .margin(20)
            .set_label_area_size(LabelAreaPosition::Left, 70)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(*self.block_counts.start()..*self.block_counts.end(), log_duration_range)?;


        time_chart
            .configure_mesh()
            .y_desc("Execution Time")
            .x_desc("Block Count")
            .y_label_formatter(&|y| ns_formatter(y))
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .draw()?;

        for depth in self.depths.clone() {
            let color = get_color(depth - self.depths.start());
            let relevant_measurements = averaged_measurements.iter().filter(|m| m.depth == depth);

            time_chart.draw_series(LineSeries::new(
                relevant_measurements.map(|m| (m.block_count, m.execution_time_ns)),
                &color,
            ))?
                .label(format!("Depth: {}", depth))
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
        }

        time_chart.configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .label_font(("sans-serif", 18).into_font())
            .draw()?;


        let game_state_footnote_text = Text::new(
            format!("N = {} game states per block count", self.number_game_states),
            (width as i32 - 10, height as i32 - 10),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Right, VPos::Bottom)),
        );
        let encoding_footnote_text = Text::new(
            format!("Encoding: {}", self.game_state_name),
            (width as i32 - 10, height as i32 - 23),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Right, VPos::Bottom)),
        );
        let cpu_footnote_text = Text::new(
            format!("CPU: {}", data.cpu_name),
            (10, height as i32 - 23),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Left, VPos::Bottom)),
        );


        time_root.draw(&game_state_footnote_text)?;
        time_root.draw(&encoding_footnote_text)?;
        time_root.draw(&cpu_footnote_text)?;

        time_root.present()?;




        let solve_root = SVGBackend::new(&solve_graph_path, (width, height)).into_drawing_area();
        solve_root.fill(&WHITE)?;

        let mut solve_chart = ChartBuilder::on(&solve_root)
            .margin(20)
            .set_label_area_size(LabelAreaPosition::Left, 50)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(*self.block_counts.start()..*self.block_counts.end(), 0.0..1.0)?;


        solve_chart
            .configure_mesh()
            .y_desc("Solved")
            .x_desc("Block Count")
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .draw()?;

        for depth in self.depths.clone() {
            let color = get_color(depth - self.depths.start());
            let relevant_measurements = averaged_measurements.iter().filter(|m| m.depth == depth);

            solve_chart.draw_series(LineSeries::new(
                relevant_measurements.map(|m| (m.block_count, m.solved)),
                &color,
            ))?
                .label(format!("Depth: {}", depth))
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
        }

        solve_chart.configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .border_style(&BLACK)
            .background_style(WHITE.filled())
            .label_font(("sans-serif", 18).into_font())
            .draw()?;


        solve_root.draw(&game_state_footnote_text)?;
        solve_root.draw(&encoding_footnote_text)?;
        solve_root.draw(&cpu_footnote_text)?;

        solve_root.present()?;

        return Ok(());
    }
}