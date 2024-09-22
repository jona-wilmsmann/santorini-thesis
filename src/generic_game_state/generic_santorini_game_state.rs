use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use anyhow::{bail, ensure, Result};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use crate::generic_game_state::GenericGameState;

#[derive(Debug)]
pub struct GenericSantoriniGameState<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> {
    pub player_a_turn: bool,
    pub player_a_workers: Option<[u8; WORKERS_PER_PLAYER]>,
    pub player_b_workers: Option<[u8; WORKERS_PER_PLAYER]>,
    pub tile_heights: [[u8; COLUMNS]; ROWS],
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> PartialEq for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn eq(&self, other: &Self) -> bool {
        let mut self_a_workers = self.player_a_workers.unwrap_or([u8::MAX; WORKERS_PER_PLAYER]);
        let mut self_b_workers = self.player_b_workers.unwrap_or([u8::MAX; WORKERS_PER_PLAYER]);
        let mut other_a_workers = other.player_a_workers.unwrap_or([u8::MAX; WORKERS_PER_PLAYER]);
        let mut other_b_workers = other.player_b_workers.unwrap_or([u8::MAX; WORKERS_PER_PLAYER]);

        self_a_workers.sort_unstable();
        self_b_workers.sort_unstable();
        other_a_workers.sort_unstable();
        other_b_workers.sort_unstable();

        return self.player_a_turn == other.player_a_turn &&
            self_a_workers == other_a_workers &&
            self_b_workers == other_b_workers &&
            self.tile_heights == other.tile_heights;
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> Eq for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> PartialOrd for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut self_a_workers = self.player_a_workers.unwrap_or([0; WORKERS_PER_PLAYER]);
        let mut self_b_workers = self.player_b_workers.unwrap_or([0; WORKERS_PER_PLAYER]);
        let mut other_a_workers = other.player_a_workers.unwrap_or([0; WORKERS_PER_PLAYER]);
        let mut other_b_workers = other.player_b_workers.unwrap_or([0; WORKERS_PER_PLAYER]);

        self_a_workers.sort_unstable();
        self_b_workers.sort_unstable();
        other_a_workers.sort_unstable();
        other_b_workers.sort_unstable();

        match self.player_a_turn.cmp(&other.player_a_turn) {
            Ordering::Equal => {
                // Then compare player A's workers
                match self_a_workers.cmp(&other_a_workers) {
                    Ordering::Equal => {
                        // Then compare player B's workers
                        match self_b_workers.cmp(&other_b_workers) {
                            Ordering::Equal => {
                                // Finally compare the tile heights
                                self.tile_heights.partial_cmp(&other.tile_heights)
                            }
                            other_order => Some(other_order),
                        }
                    }
                    other_order => Some(other_order),
                }
            }
            other_order => Some(other_order),
        }
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> Ord for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> fmt::Display for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n---Current Turn: {}---“\n", if self.player_a_turn { "A" } else { "B" })?;
        for row in (0..ROWS).rev() {
            write!(f, "---------------------\n")?;
            for column in 0..COLUMNS {
                let tile_id = row * COLUMNS + column;
                let height = self.tile_heights[row][column];
                let character = self.get_character_on_tile(tile_id);
                write!(f, "| {}{} ", height, character)?;
            }
            write!(f, "|\n")?;
        }
        write!(f, "---------------------")?;
        Ok(())
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    pub fn new(player_a_workers: Option<[u8; WORKERS_PER_PLAYER]>, player_b_workers: Option<[u8; WORKERS_PER_PLAYER]>, tile_heights: [[u8; COLUMNS]; ROWS], player_a_turn: bool) -> Result<GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER>> {
        let mut worker_tiles = Vec::with_capacity(WORKERS_PER_PLAYER * 2);

        for workers_option in [player_a_workers, player_b_workers].iter() {
            if let Some(workers) = workers_option {
                for worker_tile in workers.iter() {
                    ensure!(*worker_tile < (ROWS * COLUMNS) as u8, "Worker tile {} is out of bounds, must be less than {}", worker_tile, ROWS * COLUMNS);
                    ensure!(!worker_tiles.contains(worker_tile), "Worker tiles must be unique, {} is used multiple times", worker_tile);
                    worker_tiles.push(*worker_tile);
                }
            }
        }

        // Ensure correct tile heights
        let mut block_count = 0;
        let mut worker_on_height_3_tile = false;
        for column in 0..COLUMNS {
            for row in 0..ROWS {
                let tile_id = row * COLUMNS + column;
                let height = tile_heights[row][column];

                if worker_tiles.contains(&(tile_id as u8)) {
                    ensure!(height <= 3, "Worker tile {} cannot have a height of more than 3", tile_id);
                    if height == 3 {
                        ensure!(!worker_on_height_3_tile, "Only one worker can be on a height 3 tile");
                        worker_on_height_3_tile = true;
                    }
                } else {
                    ensure!(height <= 4, "Tile {} height must be less than or equal to 4", tile_id);
                }

                block_count += height as usize;
            }
        }

        if player_a_workers != None && player_b_workers != None {
            // Setup is complete, all workers are placed
            ensure!((block_count % 2 == 0) == player_a_turn, "It must be player A's turn if the block count is even, and player B's turn if the block count is odd");
        } else {
            // Setup is not complete, not all workers are placed
            ensure!(block_count == 0, "Block count must be 0 if workers are not placed");
            if player_a_workers == None && player_b_workers == None {
                ensure!(player_a_turn, "At the beginning of the game, player A must place their workers first");
            } else if player_a_workers != None && player_b_workers == None {
                ensure!(!player_a_turn, "After player A has placed all their workers, player B must place their workers");
            } else {
                bail!("Invalid worker placement state");
            }
        }

        return Ok(GenericSantoriniGameState {
            player_a_turn,
            player_a_workers,
            player_b_workers,
            tile_heights,
        });
    }

    pub fn set_player_a_turn(&mut self, player_a_turn: bool) {
        self.player_a_turn = player_a_turn;
    }

    pub fn get_tile_height(&self, tile_id: usize) -> u8 {
        return self.tile_heights[tile_id / COLUMNS][tile_id % COLUMNS];
    }

    pub fn has_player_a_won(&self) -> bool {
        for worker_tile in self.player_a_workers.iter().flatten() {
            let row = *worker_tile as usize / COLUMNS;
            let column = *worker_tile as usize % COLUMNS;
            if self.tile_heights[row][column] == 3 {
                return true;
            }
        }
        return false;
    }

    pub fn has_player_b_won(&self) -> bool {
        for worker_tile in self.player_b_workers.iter().flatten() {
            let row = *worker_tile as usize / COLUMNS;
            let column = *worker_tile as usize % COLUMNS;
            if self.tile_heights[row][column] == 3 {
                return true;
            }
        }
        return false;
    }

    pub fn get_character_on_tile(&self, tile_id: usize) -> char {
        if self.player_a_workers.iter().flatten().any(|&x| x as usize == tile_id) {
            return 'A';
        }
        if self.player_b_workers.iter().flatten().any(|&x| x as usize == tile_id) {
            return 'B';
        }
        return ' ';
    }
}


impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    pub fn draw_image(&self, path: &str) -> Result<()> {
        let pixels_per_tile = 100;
        let padding_between_tiles = 5;
        let border_width = 5;

        let width = COLUMNS * pixels_per_tile + (COLUMNS + 1) * padding_between_tiles;
        let height = ROWS * pixels_per_tile + (ROWS + 1) * padding_between_tiles + 50;

        let root = SVGBackend::new(path, (width as u32, height as u32)).into_drawing_area();

        for x in 0..ROWS {
            for y in 0..COLUMNS {
                let tile_id = x * COLUMNS + y;
                let height = self.get_tile_height(tile_id);
                let character = self.get_character_on_tile(tile_id);

                // Adjust x and y for drawing rectangles
                let rect_x = (y * pixels_per_tile + (y + 1) * padding_between_tiles) as i32;
                let rect_y = ((ROWS - x - 1) * pixels_per_tile + (ROWS - x) * padding_between_tiles) as i32;

                let color = match height {
                    0 => RGBColor(255, 255, 255),
                    1 => RGBColor(224, 194, 157),
                    2 => RGBColor(224, 178, 123),
                    3 => RGBColor(214, 153, 77),
                    _ => RGBColor(110, 102, 93),
                };

                let border_color = match character {
                    'A' => RGBColor(50, 59, 237),
                    'B' => RGBColor(237, 50, 75),
                    _ => RGBColor(0, 0, 0),
                };

                let text_color = BLACK;

                // Outer rectangle (border)
                let outer_rect = Rectangle::new(
                    [(rect_x, rect_y), (rect_x + pixels_per_tile as i32, rect_y + pixels_per_tile as i32)],
                    ShapeStyle {
                        color: border_color.to_rgba(),
                        filled: true,
                        stroke_width: 1,
                    },
                );

                // Inner filled rectangle
                let inner_rect = Rectangle::new(
                    [(rect_x + border_width as i32, rect_y + border_width as i32), (rect_x + (pixels_per_tile - border_width) as i32, rect_y + (pixels_per_tile - border_width) as i32)],
                    ShapeStyle {
                        color: color.to_rgba(),
                        filled: true,
                        stroke_width: 0,
                    },
                );

                // Height text in the center
                let height_text = Text::new(
                    format!("{}", height),
                    (rect_x + (pixels_per_tile / 2) as i32, rect_y + (pixels_per_tile / 2) as i32),
                    ("Arial", 40.0).into_font().color(&text_color).pos(Pos::new(HPos::Center, VPos::Center)),
                );

                // Character text in the center (if any)
                let char_text = Text::new(
                    format!("{}", character),
                    (rect_x + (pixels_per_tile / 2) as i32, rect_y + (pixels_per_tile / 2) as i32 + 25),
                    ("Arial", 30.0).into_font().color(&text_color).pos(Pos::new(HPos::Center, VPos::Center)),
                );

                root.draw(&outer_rect)?;
                root.draw(&inner_rect)?;
                root.draw(&height_text)?;
                root.draw(&char_text)?;
            }
        }

        // Draw white box behind text
        let text_background = Rectangle::new(
            [(150, height as i32 - 45), (width as i32 - 150, height as i32 - 5)],
            ShapeStyle {
                color: RGBColor(255, 255, 255).to_rgba(),
                filled: true,
                stroke_width: 0,
            },
        );
        root.draw(&text_background)?;
        // At the bottom, display the current player's turn
        let text = Text::new(
            format!("Turn: Player {}", if self.player_a_turn { "A" } else { "B" }),
            (width as i32 / 2, height as i32 - 25),
            ("Arial", 40.0).into_font().color(&BLACK).pos(Pos::new(HPos::Center, VPos::Center)),
        );
        root.draw(&text)?;

        root.present()?;

        return Ok(());
    }
}


impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn get_random_worker_positions<RNG: rand::Rng>(rng: &mut RNG) -> ([u8; WORKERS_PER_PLAYER], [u8; WORKERS_PER_PLAYER]) {
        let mut player_a_workers = [0; WORKERS_PER_PLAYER];
        let mut player_b_workers = [0; WORKERS_PER_PLAYER];

        let mut available_tiles = (0..(ROWS * COLUMNS) as u8).collect::<Vec<u8>>();

        for i in 0..(WORKERS_PER_PLAYER * 2) {
            let worker_tile_index = rng.gen_range(0..available_tiles.len());
            let worker_tile = available_tiles.swap_remove(worker_tile_index);
            if i < WORKERS_PER_PLAYER {
                player_a_workers[i] = worker_tile;
            } else {
                player_b_workers[i - WORKERS_PER_PLAYER] = worker_tile;
            }
        }

        return (player_a_workers, player_b_workers);
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericGameState for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn generate_random_state() -> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
        return Self::generate_random_state_rng(&mut rand::thread_rng());
    }

    fn generate_random_state_rng<RNG: rand::Rng>(rng: &mut RNG) -> Self {
        let mut tile_heights = [[0; COLUMNS]; ROWS];

        let (player_a_workers, player_b_workers) = GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::get_random_worker_positions(rng);
        let worker_tiles = player_a_workers.iter().chain(player_b_workers.iter()).copied().collect::<Vec<u8>>();

        let mut block_count = 0;
        for i in 0..(ROWS * COLUMNS) {
            // 2 is the maximum height for the player tiles, 4 is the maximum height for the other tiles
            let max_height = if worker_tiles.contains(&(i as u8)) { 2 } else { 4 };
            let height: usize = rng.gen_range(0..max_height + 1);
            tile_heights[i / COLUMNS][i % COLUMNS] = height as u8;
            block_count += height;
        }

        let player_a_turn = block_count % 2 == 0;

        return GenericSantoriniGameState::new(Some(player_a_workers), Some(player_b_workers), tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }

    fn generate_random_state_with_blocks(block_amount: usize) -> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
        return Self::generate_random_state_with_blocks_rng(&mut rand::thread_rng(), block_amount);
    }

    fn generate_random_state_with_blocks_rng<RNG: rand::Rng>(rng: &mut RNG, block_amount: usize) -> Self {
        assert!(block_amount <= ROWS * COLUMNS * 4 - WORKERS_PER_PLAYER * 2, "Block amount must be less than or equal to the total amount of blocks minus the workers");

        let mut tile_heights = [[0; COLUMNS]; ROWS];

        let (player_a_workers, player_b_workers) = GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::get_random_worker_positions(rng);

        let mut tile_max_heights = [[4; COLUMNS]; ROWS];
        for i in 0..WORKERS_PER_PLAYER {
            tile_max_heights[player_a_workers[i] as usize / COLUMNS][player_a_workers[i] as usize % COLUMNS] = 2;
            tile_max_heights[player_b_workers[i] as usize / COLUMNS][player_b_workers[i] as usize % COLUMNS] = 2;
        }

        let mut available_tiles = (0..(ROWS * COLUMNS) as u8).collect::<Vec<u8>>();

        for _ in 0..block_amount {
            let tile_index = rng.gen_range(0..available_tiles.len());
            let tile_id = available_tiles[tile_index];
            let tile_height = &mut tile_heights[tile_id as usize / COLUMNS][tile_id as usize % COLUMNS];
            *tile_height += 1;
            if *tile_height == tile_max_heights[tile_id as usize / COLUMNS][tile_id as usize % COLUMNS] {
                available_tiles.swap_remove(tile_index);
            }
        }

        let player_a_turn = block_amount % 2 == 0;

        return GenericSantoriniGameState::new(Some(player_a_workers), Some(player_b_workers), tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }
}