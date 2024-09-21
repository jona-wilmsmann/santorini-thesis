pub mod gs4x4_symmetric_simplified {

    /*
    12 10 9  8
    13 15 11 6
    14 3  7  5
    0  1  2  4
     */
    const TILE_ID_TO_POSITION: [usize; 16] = [0, 1, 2, 4, 14, 3, 7, 5, 13, 15, 11, 6, 12, 10, 9, 8];

    /*
    0 => 0 deg ccw
    1 => 90 deg ccw
    2 => 180 deg ccw
    3 => 270 deg ccw
    4 => 0 deg ccw + diagonal mirror
    5 => 90 deg ccw + diagonal mirror
    6 => 180 deg ccw + diagonal mirror
    7 => 270 deg ccw + diagonal mirror
     */
    type SymmetricMirrorType = u8;

    const INVALID_INDEX: usize = usize::MAX;

    #[derive(Copy, Clone)]
    pub struct SimplifiedStateVariant {
        pub(crate) player_a_position: usize,
        pub(crate) player_b_options: usize,
        pub(crate) player_b_positions: [usize; 15],
        pub(crate) total_possible_states: u64,
    }

    impl SimplifiedStateVariant {
        const fn new(player_a_tile_id: usize, player_b_tile_ids: [usize; 15]) -> Self {
            let mut player_b_options = 0;
            let mut player_b_positions = [INVALID_INDEX; 15];

            let mut i = 0;
            while i < player_b_tile_ids.len() {
                let player_b_tile_id = player_b_tile_ids[i];
                if player_b_tile_id == INVALID_INDEX {
                    break;
                }
                player_b_positions[i] = TILE_ID_TO_POSITION[player_b_tile_id];
                player_b_options += 1;
                i += 1;
            }

            let total_possible_states = player_b_options as u64 * 3u64.pow(2) * 5u64.pow(14);

            return Self {
                player_a_position: TILE_ID_TO_POSITION[player_a_tile_id],
                player_b_options,
                player_b_positions,
                total_possible_states,
            };
        }
    }
    pub(crate) const POS_TO_DIAGONALLY_MIRRORED_POS: [u64; 16] = precompute_pos_to_diagonally_mirrored_pos();
    const fn precompute_pos_to_diagonally_mirrored_pos() -> [u64; 16] {
        let mut pos_to_diagonally_mirrored_pos = [0; 16];
        let mut tile_id = 0;
        while tile_id < 16 {
            let row = tile_id / 4;
            let column = tile_id % 4;
            let diagonally_mirrored_row = column;
            let diagonally_mirrored_column = row;
            let diagonally_mirrored_tile_id = diagonally_mirrored_row * 4 + diagonally_mirrored_column;

            let pos = TILE_ID_TO_POSITION[tile_id];
            let diagonally_mirrored_pos = TILE_ID_TO_POSITION[diagonally_mirrored_tile_id];
            pos_to_diagonally_mirrored_pos[pos] = diagonally_mirrored_pos as u64;

            tile_id += 1;
        }
        return pos_to_diagonally_mirrored_pos;
    }

    const fn get_rotated_tile_id(tile_id: usize, ccw_90_rotations: usize) -> usize {
        let mut new_tile_id = tile_id;
        const ROTATION_MAP: [usize; 16] = [3, 7, 11, 15, 2, 6, 10, 14, 1, 5, 9, 13, 0, 4, 8, 12];
        let mut i = 0;
        while i < ccw_90_rotations {
            new_tile_id = ROTATION_MAP[new_tile_id];
            i += 1;
        }

        return new_tile_id;
    }

    pub const PLAYER_A_POS_PLAYER_B_POS_TO_MIRROR_TYPE: [[SymmetricMirrorType; 16]; 16] = precompute_mirror_types();

    const fn precompute_mirror_types() -> [[SymmetricMirrorType; 16]; 16] {
        let mut player_a_pos_player_b_pos_to_mirror_type = [[0; 16]; 16];

        let mut player_a_tile = 0;
        while player_a_tile < 16 {
            let player_a_pos = TILE_ID_TO_POSITION[player_a_tile];
            let mut player_b_tile = 0;
            while player_b_tile < 16 {
                let player_b_pos = TILE_ID_TO_POSITION[player_b_tile];

                let ccw_rotations = match player_a_tile {
                    0 | 1 | 4 | 5 => 0,
                    2 | 3 | 6 | 7 => 3,
                    10 | 11 | 14 | 15 => 2,
                    8 | 9 | 12 | 13 => 1,
                    _ => panic!("Invalid tile id")
                };

                let player_a_tile_rotated = get_rotated_tile_id(player_a_tile, ccw_rotations);
                let diagonal_mirroring = if player_a_tile_rotated == 4 {
                    true
                } else if player_a_tile_rotated == 1 {
                    false
                } else {
                    let player_b_tile_rotated = get_rotated_tile_id(player_b_tile, ccw_rotations);
                    match player_b_tile_rotated {
                        4 | 8 | 9 | 12 | 13 | 14 => true,
                        _ => false
                    }
                };


                let mirror_type: SymmetricMirrorType = ccw_rotations as u8 + if diagonal_mirroring { 4 } else { 0 };
                player_a_pos_player_b_pos_to_mirror_type[player_a_pos][player_b_pos] = mirror_type;

                player_b_tile += 1;
            }
            player_a_tile += 1;
        }

        return player_a_pos_player_b_pos_to_mirror_type;
    }


    pub(crate) const POSSIBLE_SIMPLIFIED_STATE_VARIANTS: [SimplifiedStateVariant; 3] = [
        SimplifiedStateVariant::new(
            0,
            [1, 2, 3, 5, 6, 7, 10, 11, 15, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX],
        ),
        SimplifiedStateVariant::new(
            1,
            [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        ),
        SimplifiedStateVariant::new(
            5,
            [0, 1, 2, 3, 6, 7, 10, 11, 15, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX, INVALID_INDEX],
        ),
    ];

    pub(crate) const SUMMED_POSSIBLE_SIMPLIFIED_STATE_OPTIONS: usize = precompute_summed_possible_simplified_state_options();
    const fn precompute_summed_possible_simplified_state_options() -> usize {
        let mut summed_possible_simplified_state_options = 0;
        let mut variant_index = 0;
        while variant_index < POSSIBLE_SIMPLIFIED_STATE_VARIANTS.len() {
            summed_possible_simplified_state_options += POSSIBLE_SIMPLIFIED_STATE_VARIANTS[variant_index].player_b_options;
            variant_index += 1;
        }
        return summed_possible_simplified_state_options;
    }
}