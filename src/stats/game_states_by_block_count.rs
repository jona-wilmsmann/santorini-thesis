use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game_state::utils::get_binomial_coefficient::calculate_binomial_coefficient;
use crate::stats::StatGenerator;

pub struct GameStatesByBlockCount {
    tiles: usize,
    workers_per_player: usize,
    graph_suffix: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameStatesByBlockCountData {
    pub tiles: usize,
    pub workers_per_player: usize,
    pub game_states_by_block_count: Vec<u128>,
}

#[derive(Debug)]
struct BlockConfiguration {
    height_4_tiles: usize,
    height_3_tiles: usize,
    height_2_tiles: usize,
    height_1_tiles: usize,
    height_0_tiles: usize,
}

impl GameStatesByBlockCount {
    pub fn new(tiles: usize, workers_per_player: usize, graph_suffix: String) -> Self {
        return GameStatesByBlockCount {
            tiles,
            workers_per_player,
            graph_suffix,
        };
    }

    fn get_block_configurations(&self) -> Vec<Vec<BlockConfiguration>> {
        let mut configurations_by_block_count = (0..=(self.tiles * 4)).map(|_| Vec::new()).collect::<Vec<Vec<BlockConfiguration>>>();

        for height_4_tiles in 0..=self.tiles {
            for height_3_tiles in 0..=(self.tiles - height_4_tiles) {
                for height_2_tiles in 0..=(self.tiles - height_4_tiles - height_3_tiles) {
                    for height_1_tiles in 0..=(self.tiles - height_4_tiles - height_3_tiles - height_2_tiles) {
                        let block_count = height_4_tiles * 4 + height_3_tiles * 3 + height_2_tiles * 2 + height_1_tiles;
                        configurations_by_block_count[block_count].push(BlockConfiguration {
                            height_4_tiles,
                            height_3_tiles,
                            height_2_tiles,
                            height_1_tiles,
                            height_0_tiles: self.tiles - height_4_tiles - height_3_tiles - height_2_tiles - height_1_tiles,
                        });
                    }
                }
            }
        }

        return configurations_by_block_count;
    }
}

impl StatGenerator for GameStatesByBlockCount {
    type DataType = GameStatesByBlockCountData;

    fn get_stat_name(&self) -> String {
        return format!("game_states_by_block_count_{}t_{}wpp", self.tiles, self.workers_per_player);
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        let mut game_states_by_block_count = vec![0u128; self.tiles * 4 + 1];

        let block_configurations_by_block_count = self.get_block_configurations();

        for block_count in 0..=self.tiles * 4 {
            let block_configurations = &block_configurations_by_block_count[block_count];

            let mut total_options = 0;

            for block_configuration in block_configurations {
                let worker_fields = block_configuration.height_2_tiles + block_configuration.height_1_tiles + block_configuration.height_0_tiles;

                let worker_position_options;
                if worker_fields >= self.workers_per_player * 2 {
                    worker_position_options = calculate_binomial_coefficient(worker_fields as u64, self.workers_per_player as u64) *
                        calculate_binomial_coefficient((worker_fields - self.workers_per_player) as u64, self.workers_per_player as u64);
                } else {
                    worker_position_options = 0;
                }

                /*
                let terminal_worker_position_options;
                if block_configuration.height_3_tiles != 0 && worker_fields > self.workers_per_player * 2 - 1 {
                    terminal_worker_position_options = block_configuration.height_3_tiles as u64 *
                        ((calculate_binomial_coefficient(worker_fields as u64, self.workers_per_player as u64 - 1) *
                            calculate_binomial_coefficient((worker_fields - self.workers_per_player + 1) as u64, self.workers_per_player as u64))
                            + (calculate_binomial_coefficient(worker_fields as u64, self.workers_per_player as u64) *
                            calculate_binomial_coefficient((worker_fields - self.workers_per_player) as u64, self.workers_per_player as u64 - 1)));
                } else {
                    terminal_worker_position_options = 0;
                }
                */
                let terminal_worker_position_options = 0;

                let height_options = calculate_binomial_coefficient(self.tiles as u64, block_configuration.height_4_tiles as u64) *
                    calculate_binomial_coefficient((self.tiles - block_configuration.height_4_tiles) as u64, block_configuration.height_3_tiles as u64) *
                    calculate_binomial_coefficient((self.tiles - block_configuration.height_4_tiles - block_configuration.height_3_tiles) as u64, block_configuration.height_2_tiles as u64) *
                    calculate_binomial_coefficient((self.tiles - block_configuration.height_4_tiles - block_configuration.height_3_tiles - block_configuration.height_2_tiles) as u64, block_configuration.height_1_tiles as u64);

                total_options += (worker_position_options as u128 + terminal_worker_position_options as u128) * height_options as u128;
            }

            game_states_by_block_count[block_count] = total_options;
        }

        return Ok(GameStatesByBlockCountData {
            tiles: self.tiles,
            workers_per_player: self.workers_per_player,
            game_states_by_block_count,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let max_game_states = *data.game_states_by_block_count.iter().max().unwrap();

        // Log scale

        let log_graph_path = format!("{}/{}-log.svg", output_folder_path, data_time);
        let root_log = SVGBackend::new(&log_graph_path, (1024, 360)).into_drawing_area();

        root_log.fill(&WHITE)?;

        let caption_log = format!("Game states by block count (Logarithmic){}", self.graph_suffix);

        let mut chart_log = ChartBuilder::on(&root_log)
            .caption(caption_log, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d((0..data.game_states_by_block_count.len() - 1).into_segmented(), (1e0..max_game_states as f64).log_scale())?;


        chart_log
            .configure_mesh()
            .disable_x_mesh()
            .y_desc("# Game States")
            .x_desc("Block Count")
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .y_label_formatter(&|y| format!("{:e}", y))
            .draw()?;


        chart_log.draw_series(
            Histogram::vertical(&chart_log)
                .style(RED.mix(0.8).filled())
                .margin(1)
                .data(data.game_states_by_block_count.iter().enumerate().map(|(i, &count)| (i, count as f64)))
        )?;

        root_log.present()?;

        // Linear scale

        let lin_graph_path = format!("{}/{}-lin.svg", output_folder_path, data_time);
        let root_lin = SVGBackend::new(&lin_graph_path, (1024, 360)).into_drawing_area();

        root_lin.fill(&WHITE)?;

        let caption_lin = format!("Game states by block count (Linear){}", self.graph_suffix);

        let mut chart_lin = ChartBuilder::on(&root_lin)
            .caption(caption_lin, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d((0..data.game_states_by_block_count.len() - 1).into_segmented(), 0f64..max_game_states as f64)?;


        chart_lin
            .configure_mesh()
            .disable_x_mesh()
            .y_desc("# Game States")
            .x_desc("Block Count")
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .y_label_formatter(&|y| format!("{:e}", y))
            .draw()?;


        chart_lin.draw_series(
            Histogram::vertical(&chart_lin)
                .style(RED.mix(0.8).filled())
                .margin(1)
                .data(data.game_states_by_block_count.iter().enumerate().map(|(i, &count)| (i, count as f64)))
        )?;

        root_log.present()?;

        return Ok(());
    }
}