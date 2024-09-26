use std::cmp::max;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use plotters::prelude::*;
use plotters::prelude::full_palette::{BROWN, DEEPPURPLE, PURPLE};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use crate::game_state::{GameState, MinimaxReady};
use crate::minimax::alpha_beta_sorted_minimax;
use crate::stats::benchmark_minimax_alpha_beta::BenchmarkMinimaxAlphaBeta;
use crate::stats::benchmark_minimax_simple::MinimaxMeasurement;
use crate::stats::formatters::{ns_formatter, value_formatter};
use crate::stats::StatGenerator;

pub struct BenchmarkMinimaxSorted<GS: GameState + MinimaxReady> {
    game_name: String,
    game_state_name: String,
    game_state_short_name: String,
    cpu_name: String,
    max_depth_sorted: usize,
    initial_game_state: GS,
    alpha_beta_benchmark: BenchmarkMinimaxAlphaBeta<GS>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BenchmarkGameStatesBasicData {
    initial_game_state: String,
    alpha_beta_measurements: Vec<MinimaxMeasurement>,
    sorted_measurements: Vec<MinimaxMeasurement>,
}


impl<GS: GameState + MinimaxReady> BenchmarkMinimaxSorted<GS> {
    pub fn new (game_name: String, game_state_name: String, game_state_short_name: String, cpu_name: String, max_depth_sorted: usize, initial_game_state: GS, alpha_beta_benchmark: BenchmarkMinimaxAlphaBeta<GS>) -> Self {
        if alpha_beta_benchmark.initial_game_state != initial_game_state {
            panic!("Initial game state for simple benchmark and alpha-beta benchmark must be the same");
        }
        return BenchmarkMinimaxSorted {
            game_name,
            game_state_name,
            game_state_short_name,
            cpu_name,
            max_depth_sorted,
            initial_game_state,
            alpha_beta_benchmark,
        };
    }
}


impl<GS: GameState + MinimaxReady> StatGenerator for BenchmarkMinimaxSorted<GS> {
    type DataType = BenchmarkGameStatesBasicData;

    fn get_stat_name(&self) -> String {
        return format!("benchmark_minimax_sorted_{}", self.game_state_short_name);
    }

    fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        if cfg!(debug_assertions) {
            return Err(anyhow::anyhow!("Benchmarking should be done in release mode"));
        }

        let mut measurements_alpha_beta = Vec::new();

        for depth in 0..=self.max_depth_sorted {
            let start = Instant::now();
            let (result, evaluated_states) = alpha_beta_sorted_minimax(&self.initial_game_state, depth);
            let computation_time = start.elapsed();
            let average_branching_factor = (evaluated_states as f32).powf(1.0 / (depth as f32));
            measurements_alpha_beta.push(MinimaxMeasurement {
                depth,
                computation_time,
                evaluated_states,
                average_branching_factor,
                result,
            });
        }

        let alpha_beta_data_file = self.alpha_beta_benchmark.get_most_recent_data_file()?;
        let alpha_beta_data = self.alpha_beta_benchmark.get_data(&alpha_beta_data_file)?;

        return Ok(BenchmarkGameStatesBasicData {
            initial_game_state: self.initial_game_state.to_string(),
            alpha_beta_measurements: alpha_beta_data.alpha_beta_measurements,
            sorted_measurements: measurements_alpha_beta,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let height = 1000;
        let width = 500;

        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);
        let root = SVGBackend::new(&graph_path, (height, width)).into_drawing_area();

        root.fill(&WHITE)?;


        let simple_max_measurement_time_ns = data.alpha_beta_measurements.iter().map(|m| m.computation_time).max().unwrap().as_nanos() as usize;
        let simple_max_evaluated_states = data.alpha_beta_measurements.iter().map(|m| m.evaluated_states).max().unwrap();
        // let alpha_beta_max_measurement_time_ns = data.alpha_beta_measurements.iter().map(|m| m.computation_time).max().unwrap().as_nanos() as usize;
        // let alpha_beta_max_evaluated_states = data.alpha_beta_measurements.iter().map(|m| m.evaluated_states).max().unwrap();

        let max_depth = max(self.alpha_beta_benchmark.max_depth_alpha_beta, self.max_depth_sorted);

        let max_upper_bound = max(simple_max_measurement_time_ns, simple_max_evaluated_states);
        let rounded_upper_bound = 10usize.pow((max_upper_bound as f32).log10().ceil() as u32);

        let duration_range = 0..rounded_upper_bound;
        let log_duration_range = duration_range.log_scale();

        let evaluated_states_range = 0..rounded_upper_bound;
        let log_evaluated_states_range = evaluated_states_range.log_scale();

        let x_spec = (0..max_depth).into_segmented();


        // Line chart showing computation time, bar chart showing evaluated states
        let mut chart = ChartBuilder::on(&root)
            .caption(format!("Sorted Alpha Beta Minimax Benchmark for {}", self.game_name), ("sans-serif", 30).into_font())
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

        for (measurements, color, point_offset, label, draw_points) in [
            (data.alpha_beta_measurements, RED, (-15, -15), "Alpha-Beta Minimax", false),
            (data.sorted_measurements, DEEPPURPLE, (-15, 8), "Sorted Alpha-Beta Minimax", true)
        ].into_iter() {
            // draw line series
            chart.draw_series(LineSeries::new(
                measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.computation_time.as_nanos() as usize)),
                &color,
            ))?
                .label(label)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x - 15, y)], &color));

            // add value labels to each point (only show measurement time formatted using ns_formatter)
            if draw_points {
                chart.draw_series(PointSeries::of_element(
                    measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.computation_time.as_nanos() as usize)),
                    5,
                    &color,
                    &|c, s, st| {
                        return EmptyElement::at(c.clone())
                            + Circle::new((0, 0), s, st.filled())
                            + Text::new(ns_formatter(&c.1), point_offset, ("sans-serif", 15).into_font().color(&color));
                    },
                ))?;
            } else {
                chart.draw_series(PointSeries::of_element(
                    measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.computation_time.as_nanos() as usize)),
                    5,
                    &color,
                    &|c, s, st| {
                        return EmptyElement::at(c.clone())
                            + Circle::new((0, 0), s, st.filled())
                    },
                ))?;
            }

            // bar chart of evaluated states
            chart.draw_secondary_series(
                Histogram::vertical(&chart)
                    .style(color.mix(0.5).filled())
                    .margin(1)
                    .data(measurements.iter().map(|m| (m.depth, m.evaluated_states))),
            )?;


            if draw_points {
                // add value labels to each point (only show measurement time formatted using ns_formatter)
                chart.draw_secondary_series(PointSeries::of_element(
                    measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.evaluated_states)),
                    5,
                    &color,
                    &|c, _s, _st| {
                        return EmptyElement::at(c.clone())
                            + Text::new(value_formatter(&c.1), (value_formatter(&c.1).len() as i32 * -4, 4), ("sans-serif", 17).into_font());
                    },
                ))?;
            }
        }

        chart.configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .margin(20)
            .legend_area_size(5)
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.7))
            .label_font(("sans-serif", 18).into_font())
            .draw()?;



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