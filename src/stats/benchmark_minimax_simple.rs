use core::hint::black_box;
use std::cmp::max;
use std::time::{Duration, Instant};
use num_format::Locale::ro;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use crate::game_state::game_state_4x4_binary_3bit::GameState4x4Binary3Bit;
use crate::game_state::game_state_4x4_binary_4bit::GameState4x4Binary4Bit;
use crate::game_state::game_state_4x4_struct::GameState4x4Struct;
use crate::game_state::{GameState, MinimaxReady};
use crate::generic_game_state::generic_santorini_game_state::GenericSantoriniGameState;
use crate::generic_game_state::GenericGameState;
use crate::minimax::simple_minimax;
use crate::stats::formatters::{ns_formatter, value_formatter};
use crate::stats::StatGenerator;

pub struct BenchmarkMinimaxSimple<GS: GameState + MinimaxReady> {
    game_name: String,
    game_state_name: String,
    game_state_short_name: String,
    cpu_name: String,
    max_depth: usize,
    initial_game_state: GS,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimaxMeasurement {
    depth: usize,
    computation_time: Duration,
    evaluated_states: usize,
    average_branching_factor: f32,
    result: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchmarkGameStatesBasicData {
    initial_game_state: String,
    measurements: Vec<MinimaxMeasurement>,
}


impl<GS: GameState + MinimaxReady> BenchmarkMinimaxSimple<GS> {
    pub fn new (game_name: String, game_state_name: String, game_state_short_name: String, cpu_name: String, max_depth: usize, initial_game_state: GS) -> Self {
        return BenchmarkMinimaxSimple {
            game_name,
            game_state_name,
            game_state_short_name,
            cpu_name,
            max_depth,
            initial_game_state,
        };
    }
}


impl<GS: GameState + MinimaxReady> StatGenerator for BenchmarkMinimaxSimple<GS> {
    type DataType = BenchmarkGameStatesBasicData;

    fn get_stat_name(&self) -> String {
        return format!("benchmark_minimax_simple_{}", self.game_state_short_name);
    }

    fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        if cfg!(debug_assertions) {
            return Err(anyhow::anyhow!("Benchmarking should be done in release mode"));
        }

        let mut measurements = Vec::new();

        for depth in 0..=self.max_depth {
            let start = Instant::now();
            let (result, evaluated_states) = simple_minimax(&self.initial_game_state, depth);
            let computation_time = start.elapsed();
            let average_branching_factor = (evaluated_states as f32).powf(1.0 / (depth as f32));
            measurements.push(MinimaxMeasurement {
                depth,
                computation_time,
                evaluated_states,
                average_branching_factor,
                result,
            });
        }

        return Ok(BenchmarkGameStatesBasicData {
            initial_game_state: self.initial_game_state.to_string(),
            measurements,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let max_measurement_time_ns = data.measurements.iter().map(|m| m.computation_time).max().unwrap().as_nanos() as usize;
        let max_evaluated_states = data.measurements.iter().map(|m| m.evaluated_states).max().unwrap();

        let height = 1000;
        let width = 500;

        let graph_path = format!("{}/{}.svg", output_folder_path, self.get_stat_name());
        let root = SVGBackend::new(&graph_path, (height, width)).into_drawing_area();

        root.fill(&WHITE)?;


        let max_upper_bound = max(max_measurement_time_ns, max_evaluated_states);
        let rounded_upper_bound = 10usize.pow((max_upper_bound as f32).log10().ceil() as u32);

        let duration_range = 0..rounded_upper_bound;
        let log_duration_range = duration_range.log_scale();

        let evaluated_states_range = 0..rounded_upper_bound;
        let log_evaluated_states_range = evaluated_states_range.log_scale();

        let x_spec = (0..self.max_depth).into_segmented();


        // Line chart showing computation time, bar chart showing evaluated states
        let mut chart = ChartBuilder::on(&root)
            .caption(format!("Simple Minimax Benchmark for {}", self.game_name), ("sans-serif", 30).into_font())
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 70)
            .set_label_area_size(LabelAreaPosition::Right, 70)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(x_spec.clone(), log_duration_range)?
            .set_secondary_coord(x_spec, log_evaluated_states_range);

        chart
            .configure_mesh()
            .y_desc("Execution Time")
            .x_desc("Depth")
            .y_label_formatter(&|y| ns_formatter(y))
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .draw()?;

        chart
            .configure_secondary_axes()
            .y_desc("Evaluated States")
            .y_label_formatter(&|y| value_formatter(y))
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .draw()?;

        // draw line series
        chart.draw_series(LineSeries::new(
            data.measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.computation_time.as_nanos() as usize)),
            &BLUE,
        ))?;

        // add value labels to each point (only show measurement time formatted using ns_formatter)
        chart.draw_series(PointSeries::of_element(
            data.measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.computation_time.as_nanos() as usize)),
            5,
            &BLUE,
            &|c, s, st| {
                return EmptyElement::at(c.clone())
                    + Circle::new((0, 0), s, st.filled())
                    + Text::new(ns_formatter(&c.1), (10, 5), ("sans-serif", 15).into_font());
            },
        ))?;

        // bar chart of evaluated states
        chart.draw_secondary_series(
            Histogram::vertical(&chart)
                .style(BLUE.mix(0.5).filled())
                .margin(1)
                .data(data.measurements.iter().map(|m| (m.depth, m.evaluated_states))),
        )?;



        // add value labels to each point (only show measurement time formatted using ns_formatter)
        chart.draw_secondary_series(PointSeries::of_element(
            data.measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.evaluated_states)),
            5,
            &BLUE,
            &|c, s, st| {
                return EmptyElement::at(c.clone())
                    + Text::new(value_formatter(&c.1), (value_formatter(&c.1).len() as i32 * -4, 4), ("sans-serif", 17).into_font());
            },
        ))?;


        // add footnote in bottom right corner
        let game_state_footnote_text = Text::new(
            format!("Game State: {}", self.game_state_name),
            (height as i32 - 10, width as i32 - 10),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Right, VPos::Bottom))
        );
        root.draw(&game_state_footnote_text)?;

        // add footnote in bottom left corner
        let cpu_footnote_text = Text::new(
            format!("CPU: {}", self.cpu_name),
            (10, width as i32 - 10),
            ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Left, VPos::Bottom))
        );
        root.draw(&cpu_footnote_text)?;


        root.present()?;

        return Ok(());
    }
}