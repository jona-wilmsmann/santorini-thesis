use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait StatGenerator {
    type DataType: Serialize + for<'a> Deserialize<'a>;

    fn get_stat_name(&self) -> String;
    fn gather_data(&self) -> Result<Self::DataType>;
    fn generate_graph(&self, data: Self::DataType, data_time: String, output_folder_path: &str) -> Result<()>;


    fn store_data(&self, data: Self::DataType) -> Result<String> {
        let data_folder = format!("stats/data/{}", self.get_stat_name());
        std::fs::create_dir_all(data_folder.clone())?;
        let file_name = format!("{}.json", chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        let file_path = format!("{}/{}", data_folder, file_name);

        let data = serde_json::to_string(&data)?;
        std::fs::write(file_path.clone(), data)?;

        return Ok(file_name);
    }

    fn get_data(&self, file_name: &str) -> Result<Self::DataType> {
        let data = std::fs::read_to_string(format!("stats/data/{}/{}", self.get_stat_name(), file_name))?;
        let data: Self::DataType = serde_json::from_str(&data)?;
        return Ok(data);
    }

    fn get_most_recent_data_file(&self) -> Result<String> {
        let data_folder = format!("stats/data/{}", self.get_stat_name());
        // Find all files in the data folder
        let files = std::fs::read_dir(data_folder.clone())?;
        let mut most_recent_file = None;
        for file in files {
            let file = file?;
            let file_name = file.file_name().to_str().ok_or(anyhow::anyhow!("Invalid file name"))?.to_string();

            if let Some(current_most_recent_file) = most_recent_file.clone() {
                if file_name > current_most_recent_file {
                    most_recent_file = Some(file_name);
                }
            } else {
                most_recent_file = Some(file_name);
            }
        }

        return if let Some(most_recent_file) = most_recent_file {
            Ok(most_recent_file)
        } else {
            Err(anyhow::anyhow!("No data files found in {}", data_folder))
        }
    }

    fn gather_and_store_data(&self) -> Result<String> {
        let data = self.gather_data()?;
        return self.store_data(data);
    }

    fn generate_graph_from_most_recent_data(&self) -> Result<()> {
        let most_recent_data_file = self.get_most_recent_data_file()?;
        let data = self.get_data(&most_recent_data_file)?;
        // Strip file ending
        let data_time = most_recent_data_file.split('.').next().unwrap().to_string();
        let output_folder_path = format!("stats/graphs/{}", self.get_stat_name());
        std::fs::create_dir_all(output_folder_path.clone())?;
        return self.generate_graph(data, data_time, output_folder_path.as_str());
    }
}

pub mod formatters;
pub mod game_states_by_block_count;
pub mod branching_factor_by_block_count;
pub mod benchmark_minimax_simple;
pub mod benchmark_minimax_alpha_beta;
pub mod benchmark_minimax_sorted;