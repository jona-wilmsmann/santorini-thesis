// These heuristics are meant to be used solely for regular, 5x5 Santorini

pub const NO_NEIGHBOR: usize = usize::MAX;
pub const TILE_TO_NEIGHBORS: [[usize; 8]; 25] = precompute_tile_to_neighbors();
const fn precompute_tile_to_neighbors() -> [[usize; 8]; 25] {
    let mut position_to_neighbors = [[NO_NEIGHBOR; 8]; 25];

    let mut row: isize = 0;
    while row < 5 {
        let mut column = 0;
        while column < 5 {
            let tile_id = (row * 5 + column) as usize;
            let mut position_neighbor_index = 0;

            let mut neighbor_row = row - 1;
            while neighbor_row <= row + 1 {
                if neighbor_row < 0 || neighbor_row >= 5 {
                    neighbor_row += 1;
                    continue;
                }
                let mut neighbor_column = column - 1;
                while neighbor_column <= column + 1 {
                    if neighbor_column < 0 || neighbor_column >= 5 || (neighbor_row == row && neighbor_column == column) {
                        neighbor_column += 1;
                        continue;
                    }
                    let neighbor_tile_id = (neighbor_row * 5 + neighbor_column) as usize;
                    position_to_neighbors[tile_id][position_neighbor_index] = neighbor_tile_id;
                    position_neighbor_index += 1;

                    neighbor_column += 1;
                }
                neighbor_row += 1;
            }
            column += 1;
        }
        row += 1;
    }
    return position_to_neighbors;
}

pub mod boreham_heuristic;
pub mod boreham_greedy_heuristic;