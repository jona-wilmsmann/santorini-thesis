use std::time::Duration;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use serde::{Deserialize, Serialize};
use crate::stats::utils::formatters::{ns_formatter, value_formatter};

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimaxMeasurement {
    pub depth: usize,
    pub computation_time: Duration,
    pub evaluated_states: usize,
    pub average_branching_factor: f32,
    pub result: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AverageMinimaxMeasurement {
    pub depth: usize,
    pub computation_time: Duration,
    pub evaluated_states: usize,
    pub average_branching_factor: f32,
    pub solve_portion: f32,
}

pub struct MinimaxBenchmarkData {
    pub label: String,
    pub cpu_name: String,
    pub color: RGBColor,
    pub draw_execution_time_text: bool,
    pub draw_game_states_text: bool,
    pub average_measurements: Vec<AverageMinimaxMeasurement>,
    pub raw_measurements: Vec<Vec<MinimaxMeasurement>>
}

pub fn draw_minimax_benchmark(graph_path: String, graph_name: String, encoding_name: String, data: Vec<MinimaxBenchmarkData>) -> anyhow::Result<()> {
    let width = 1000;
    let height = 500;

    let root = SVGBackend::new(&graph_path, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let graph_upper_bound = 1e11 as usize;

    let log_duration_range = (0..graph_upper_bound).log_scale();
    let log_evaluated_states_range = (0..graph_upper_bound).log_scale();

    let max_depth = data.iter().map(|d| d.average_measurements.len()).max().expect("No benchmarks found") - 1;
    let x_spec = (0..max_depth).into_segmented();


    // Line chart showing computation time, bar chart showing evaluated states
    let mut chart = ChartBuilder::on(&root)
        .caption(graph_name, ("sans-serif", 30).into_font())
        .margin(10)
        .set_label_area_size(LabelAreaPosition::Left, 70)
        .set_label_area_size(LabelAreaPosition::Right, 70)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
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


    let mut number_of_measurements_option = None;
    let mut block_count_option = None;
    let mut cpu_name_option = None;
    for benchmark in &data {
        if let Some(number_of_measurements) = number_of_measurements_option {
            assert_eq!(number_of_measurements, benchmark.raw_measurements.len());
        } else {
            number_of_measurements_option = Some(benchmark.raw_measurements.len());
        }

        if let Some(block_count) = block_count_option {
            assert_eq!(block_count, benchmark.average_measurements.len() + 1);
        } else {
            block_count_option = Some(benchmark.average_measurements.len() + 1);
        }

        if let Some(ref cpu_name) = cpu_name_option {
            assert_eq!(*cpu_name, benchmark.cpu_name);
        } else {
            cpu_name_option = Some(benchmark.cpu_name.clone());
        }
    }
    let number_of_measurements = number_of_measurements_option.expect("No benchmarks found");
    let block_count = block_count_option.expect("No benchmarks found");
    let cpu_name = cpu_name_option.expect("No benchmarks found");

    for benchmark in &data {
        let max_depth = benchmark.average_measurements.len() - 1;

        let mut raw_execution_times_simple = vec![Vec::with_capacity(number_of_measurements); max_depth + 1];
        for depth in 0..=max_depth {
            for single_measurement in benchmark.raw_measurements.iter() {
                raw_execution_times_simple[depth].push(single_measurement[depth].computation_time.as_nanos() as usize);
            }
            raw_execution_times_simple[depth].sort();
        }

        let errors = raw_execution_times_simple.iter().map(|execution_times| {
            let p5 = execution_times[execution_times.len() / 20];
            let p95 = execution_times[19 * execution_times.len() / 20];
            let average = execution_times.iter().sum::<usize>() / execution_times.len();
            return (p5, average, p95);
        }).collect::<Vec<(usize, usize, usize)>>();


        // draw line series
        chart.draw_series(LineSeries::new(
            benchmark.average_measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.computation_time.as_nanos() as usize)),
            &benchmark.color,
        ))?
            .label(&benchmark.label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &benchmark.color));

        // draw error bars
        chart.draw_series(errors.iter().enumerate().map(|(i, (p5, average, p95))| {
            return ErrorBar::new_vertical(
                SegmentValue::CenterOf(i),
                *p5,
                *average,
                *p95,
                benchmark.color.filled(),
                10
            );
        }))?;

        if benchmark.draw_execution_time_text {
            // add value labels to each point (only show measurement time formatted using ns_formatter)
            chart.draw_series(PointSeries::of_element(
                benchmark.average_measurements.iter().map(|m| (SegmentValue::CenterOf(m.depth), m.computation_time.as_nanos() as usize)),
                5,
                &benchmark.color,
                &|c, _s, _st| {
                    return EmptyElement::at(c.clone())
                        + Text::new(ns_formatter(&c.1), (10, 5), ("sans-serif", 15).into_font().color(&benchmark.color));
                },
            ))?;
        }

        // bar chart of evaluated states
        chart.draw_secondary_series(
            Histogram::vertical(&chart)
                .style(benchmark.color.mix(0.4).filled())
                .margin(10)
                .data(benchmark.average_measurements.iter().map(|m| (m.depth, m.evaluated_states))),
        )?;


        if benchmark.draw_execution_time_text {
            // add value labels to each point (only show measurement time formatted using ns_formatter)
            chart.draw_secondary_series(PointSeries::of_element(
                benchmark.average_measurements.iter().take(1).map(|m| (SegmentValue::CenterOf(m.depth), m.evaluated_states)),
                5,
                &benchmark.color,
                &|c, _s, _st| {
                    return EmptyElement::at(c.clone())
                        + Text::new(value_formatter(&c.1), (value_formatter(&c.1).len() as i32 * -4, -15), ("sans-serif", 17).into_font());
                },
            ))?;
            chart.draw_secondary_series(PointSeries::of_element(
                benchmark.average_measurements.iter().skip(1).map(|m| (SegmentValue::CenterOf(m.depth), m.evaluated_states)),
                5,
                &benchmark.color,
                &|c, _s, _st| {
                    return EmptyElement::at(c.clone())
                        + Text::new(value_formatter(&c.1), (value_formatter(&c.1).len() as i32 * -4, 6), ("sans-serif", 17).into_font());
                },
            ))?;
        }
    }

    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .label_font(("sans-serif", 18).into_font())
        .draw()?;


    // add footnote in bottom right corner
    let game_state_footnote_text = Text::new(
        format!("N = {} game states, Starting block count = {}", number_of_measurements, block_count),
        (width as i32 - 10, height as i32 - 10),
        ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Right, VPos::Bottom)),
    );
    root.draw(&game_state_footnote_text)?;
    let encoding_footnote_text = Text::new(
        format!("Encoding: {}", encoding_name),
        (width as i32 - 10, height as i32 - 23),
        ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Right, VPos::Bottom)),
    );
    root.draw(&encoding_footnote_text)?;

    // add footnote in bottom left corner

    let cpu_footnote_text = Text::new(
        format!("CPU: {}", cpu_name),
        (10, height as i32 - 23),
        ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Left, VPos::Bottom)),
    );
    root.draw(&cpu_footnote_text)?;

    let error_explainer_text = Text::new(
        "Central line is average value, error bars indicate 5th and 95th percentiles".to_string(),
        (10, height as i32 - 10),
        ("Arial", 12).into_font().color(&BLACK).pos(Pos::new(HPos::Left, VPos::Bottom)),
    );
    root.draw(&error_explainer_text)?;


    root.present()?;

    return Ok(());
}