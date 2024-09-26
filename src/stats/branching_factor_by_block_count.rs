use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use crate::game_state::GameState;
use crate::generic_game_state::GenericGameState;
use crate::stats::StatGenerator;

pub struct BranchingFactorByBlockCount<GS: GameState> {
    _marker: std::marker::PhantomData<GS>,
    game_name: String,
    graph_suffix: String,
    max_block_count: usize,
    max_graph_block_count: usize,
    sample_size_per_block_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BranchingFactorByBlockCountData {
    sample_size_per_block_count: usize,
    average_branching_factor_by_block_count: Vec<f32>,
}

impl<GS: GameState> BranchingFactorByBlockCount<GS> {

    pub fn new(game_name: String, graph_suffix: String, max_block_count: usize, max_graph_block_count: usize, sample_size_per_block_count: usize) -> Self {
        return BranchingFactorByBlockCount {
            _marker: std::marker::PhantomData,
            game_name,
            graph_suffix,
            max_block_count,
            max_graph_block_count,
            sample_size_per_block_count,
        };
    }

}

impl<GS: GameState> StatGenerator for BranchingFactorByBlockCount<GS> {
    type DataType = BranchingFactorByBlockCountData;

    fn get_stat_name(&self) -> String {
        return format!("branching_factor_by_block_count_{}", self.game_name);
    }

    async fn gather_data(&self) -> anyhow::Result<Self::DataType> {
        let mut rng = rand::thread_rng();
        let mut children_states = Vec::new();

        let mut average_branching_factor_by_block_count = Vec::with_capacity(self.max_graph_block_count + 1);

        for block_count in 0..=self.max_graph_block_count {
            if block_count > self.max_block_count {
                average_branching_factor_by_block_count.push(0f32);
                continue;
            }

            let mut summed_branching_factors: usize = 0;
            for _ in 0..self.sample_size_per_block_count {
                let random_generic_state = GS::GenericGameState::generate_random_state_with_blocks_rng(&mut rng, block_count);
                let random_state = GS::from_generic_game_state(&random_generic_state);
                random_state.get_children_states_reuse_vec(&mut children_states);
                summed_branching_factors += children_states.len();
            }
            average_branching_factor_by_block_count.push(summed_branching_factors as f32 / self.sample_size_per_block_count as f32);
        }

        return Ok(BranchingFactorByBlockCountData {
            sample_size_per_block_count: self.sample_size_per_block_count,
            average_branching_factor_by_block_count,
        });
    }

    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> anyhow::Result<()> {
        let max_branching_factor = *data.average_branching_factor_by_block_count.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
        let ceiled_max_branching_factor = (max_branching_factor / 10.0).ceil() * 10.0;

        let graph_path = format!("{}/{}.svg", output_folder_path, data_time);
        let root = SVGBackend::new(&graph_path, (1024, 360)).into_drawing_area();

        root.fill(&WHITE)?;

        let caption = format!("Branching factor by block count{}", self.graph_suffix);

        let mut chart_log = ChartBuilder::on(&root)
            .caption(caption, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d((0..data.average_branching_factor_by_block_count.len() - 1).into_segmented(), 0f32..ceiled_max_branching_factor)?;


        chart_log
            .configure_mesh()
            .disable_x_mesh()
            .y_desc("Average Branching Factor")
            .x_desc("Block Count")
            .axis_desc_style(("sans-serif", 20).into_font())
            .label_style(("sans-serif", 15).into_font())
            .draw()?;


        chart_log.draw_series(
            Histogram::vertical(&chart_log)
                .style(RED.mix(0.8).filled())
                .margin(1)
                .data(data.average_branching_factor_by_block_count.iter().enumerate().map(|(i, &branching_factor)| (i, branching_factor)))
        )?;

        root.present()?;

        return Ok(());
    }
}