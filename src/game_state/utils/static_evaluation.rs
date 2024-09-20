pub mod gs4x4 {
    /*
    12 10 9  8
    13 15 11 6
    14 3  7  5
    0  1  2  4
     */
    const TILE_ID_TO_POSITION: [usize; 16] = [0, 1, 2, 4, 14, 3, 7, 5, 13, 15, 11, 6, 12, 10, 9, 8];

    const DISTANCE_TO_STATIC_VALUATION: [f32; 4] = [5.0, 2.0, 1.0, 0.5];
    const HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION: [[f32; 5]; 3] = [
        [1.0, 1.5, -1.0, 0.0, -1.0], //Start height 0
        [1.0, 1.5, 2.0, 0.5, -1.0], //Start height 1
        [1.0, 1.5, 2.0, 3.0, -1.0], //Start height 2
    ];

    const POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION: [[[[f32; 5]; 3]; 16]; 16] =
        precompute_position_to_position_to_height_to_height_to_valuation();
    const fn precompute_position_to_position_to_height_to_height_to_valuation() -> [[[[f32; 5]; 3]; 16]; 16] {
        let mut position_to_position_to_height_to_height_to_valuation = [[[[0.0; 5]; 3]; 16]; 16];

        let mut i = 0;
        while i < 16 {
            let row_i = i / 4;
            let column_i = i % 4;
            let position_i = TILE_ID_TO_POSITION[i];
            let mut j = 0;
            while j < 16 {
                let row_j = j / 4;
                let column_j = j % 4;
                let position_j = TILE_ID_TO_POSITION[j];

                let row_distance = if row_i > row_j { row_i - row_j } else { row_j - row_i };
                let column_distance = if column_i > column_j { column_i - column_j } else { column_j - column_i };
                let distance = if row_distance > column_distance { row_distance } else { column_distance };

                let mut start_height = 0;
                while start_height <= 2 {
                    let mut neighbor_height = 0;
                    while neighbor_height <= 4 {
                        let height_valuation = HEIGHT_TO_NEIGHBOR_HEIGHT_TO_STATIC_VALUATION[start_height][neighbor_height];
                        let distance_valuation = DISTANCE_TO_STATIC_VALUATION[distance];
                        position_to_position_to_height_to_height_to_valuation[position_i][position_j][start_height][neighbor_height] = height_valuation * distance_valuation;
                        neighbor_height += 1;
                    }

                    start_height += 1;
                }
                j += 1;
            }
            i += 1;
        }

        return position_to_position_to_height_to_height_to_valuation;
    }


    // Positions are 0-15 if placed, or 16 if not placed
    pub fn get_static_evaluation(position_heights: [u8; 16], player_a_position: usize, player_b_position: usize, player_a_turn: bool) -> f32 {
        if player_a_position < 16 && position_heights[player_a_position] == 3 {
            return if player_a_turn { f32::MAX } else { f32::MIN };
        } else if player_b_position < 16 && position_heights[player_b_position] == 3 {
            return if player_a_turn { f32::MIN } else { f32::MAX };
        }

        let mut valuation = 0.0;

        // TODO: Consider whose turn it is
        // TODO: Consider unplaced workers

        for i in 0..16 {
            valuation += POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[player_a_position][i][position_heights[player_a_position] as usize][position_heights[i] as usize];
            valuation -= POSITION_TO_POSITION_TO_HEIGHT_TO_HEIGHT_TO_VALUATION[player_b_position][i][position_heights[player_b_position] as usize][position_heights[i] as usize];
        }

        return valuation;
    }
}