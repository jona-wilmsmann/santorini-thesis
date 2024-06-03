pub const fn precompute_position_to_tile_id(tile_id_to_position: [usize; 16]) -> [usize; 16] {
    let mut position_to_tile_id = [0; 16];
    let mut i = 0;
    while i < 16 {
        let position = tile_id_to_position[i];
        position_to_tile_id[position] = i;
        i += 1;
    }
    return position_to_tile_id;
}