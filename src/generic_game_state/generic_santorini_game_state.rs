use std::fmt;
use std::fmt::Formatter;
use anyhow::{ensure, Result};
use plotters::element::{ComposedElement, Drawable};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use crate::generic_game_state::GenericGameState;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct GenericSantoriniGameState<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> {
    pub player_a_turn: bool,
    pub player_a_workers: [u8; WORKERS_PER_PLAYER],
    pub player_b_workers: [u8; WORKERS_PER_PLAYER],
    pub tile_heights: [[u8; COLUMNS]; ROWS],
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

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> fmt::Debug for GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<const ROWS: usize, const COLUMNS: usize, const WORKERS_PER_PLAYER: usize> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
    pub const WORKER_NOT_PLACED: u8 = u8::MAX;

    pub fn new(player_a_workers: [u8; WORKERS_PER_PLAYER], player_b_workers: [u8; WORKERS_PER_PLAYER], tile_heights: [[u8; COLUMNS]; ROWS], player_a_turn: bool) -> Result<GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER>> {
        let mut player_a_worker_count = 0;
        let mut player_b_worker_count = 0;


        for (worker_count, workers) in [(&mut player_a_worker_count, &player_a_workers), (&mut player_b_worker_count, &player_b_workers)] {
            let mut last_worker_not_placed = false;

            for worker in workers {
                if *worker == Self::WORKER_NOT_PLACED {
                    last_worker_not_placed = true;
                    continue;
                }
                ensure!(!last_worker_not_placed, "Player workers must be placed in order");
                *worker_count += 1;
            }
        }

        // TODO: At the beginning of the game, player A must place all their workers, then player B must place all their workers
        // The current implementation has them taking alternating turns, which is incorrect.

        if player_a_worker_count == 0 && player_b_worker_count == 0 {
            // Beginning of the game, no workers are placed
            ensure!(player_a_turn, "At the beginning of the game, player A must place the first worker");
        }

        if player_a_worker_count != WORKERS_PER_PLAYER || player_b_worker_count != WORKERS_PER_PLAYER {
            // Beginning of the game, not all workers are placed
            if player_a_turn {
                ensure!(player_a_worker_count == player_b_worker_count, "During worker placement, if it is player A's turn, both players must have the same amount of workers placed");
            } else {
                ensure!(player_a_worker_count == player_b_worker_count + 1, "During worker placement, if it is player B's turn, player A must have one more worker placed than player B");
            }

            for column in 0..COLUMNS {
                for row in 0..ROWS {
                    ensure!(tile_heights[column][row] == 0, "At the beginning of the game, before all workers are placed, all heights must be 0");
                }
            }
        } else {
            // Normal game state, all workers are placed
            for column in 0..COLUMNS {
                for row in 0..ROWS {
                    ensure!(tile_heights[column][row] <= 4, "Tile {},{} height must be less than or equal to 4", column, row);
                }
            }
        }


        let mut worker_tiles = Vec::with_capacity(WORKERS_PER_PLAYER * 2);

        let mut worker_on_height_3_tile = false;
        for worker_tile in player_a_workers.iter().chain(player_b_workers.iter()) {
            if *worker_tile == Self::WORKER_NOT_PLACED {
                continue;
            }

            ensure!(*worker_tile < (ROWS * COLUMNS) as u8, "Worker tile {} is out of bounds, must be less than {}", worker_tile, ROWS * COLUMNS);

            let worker_tile_height = tile_heights[*worker_tile as usize / COLUMNS][*worker_tile as usize % COLUMNS];

            ensure!(worker_tile_height < 4, "Worker tile {} cannot have a height of 4", worker_tile);

            if worker_tile_height == 3 {
                ensure!(!worker_on_height_3_tile, "Only one worker can be on a height 3 tile");
                worker_on_height_3_tile = true;
            }

            ensure!(!worker_tiles.contains(worker_tile), "Worker tiles must be unique, {} is used multiple times", worker_tile);
            worker_tiles.push(*worker_tile);
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

    pub fn get_character_on_tile(&self, tile_id: usize) -> char {
        return if self.player_a_workers.contains(&(tile_id as u8)) {
            'A'
        } else if self.player_b_workers.contains(&(tile_id as u8)) {
            'B'
        } else {
            ' '
        };
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
        let mut player_a_workers = [GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::WORKER_NOT_PLACED; WORKERS_PER_PLAYER];
        let mut player_b_workers = [GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::WORKER_NOT_PLACED; WORKERS_PER_PLAYER];

        let mut worker_tiles = Vec::with_capacity(WORKERS_PER_PLAYER * 2);

        for i in 0..(WORKERS_PER_PLAYER * 2) {
            let mut worker_tile;
            loop {
                worker_tile = rng.gen_range(0..(ROWS * COLUMNS)) as u8;
                if !worker_tiles.contains(&worker_tile) {
                    break;
                }
            }
            if i < WORKERS_PER_PLAYER {
                player_a_workers[i] = worker_tile;
            } else {
                player_b_workers[i - WORKERS_PER_PLAYER] = worker_tile;
            }
            worker_tiles.push(worker_tile);
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

        return GenericSantoriniGameState::new(player_a_workers, player_b_workers, tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }

    fn generate_random_state_with_blocks(block_amount: usize) -> GenericSantoriniGameState<ROWS, COLUMNS, WORKERS_PER_PLAYER> {
        return Self::generate_random_state_with_blocks_rng(&mut rand::thread_rng(), block_amount);
    }

    fn generate_random_state_with_blocks_rng<RNG: rand::Rng>(rng: &mut RNG, block_amount: usize) -> Self {
        let mut tile_heights = [[0; COLUMNS]; ROWS];

        let (player_a_workers, player_b_workers) = GenericSantoriniGameState::<ROWS, COLUMNS, WORKERS_PER_PLAYER>::get_random_worker_positions(rng);

        let mut tile_max_heights = [[4; COLUMNS]; ROWS];
        for i in 0..WORKERS_PER_PLAYER {
            tile_max_heights[player_a_workers[i] as usize / COLUMNS][player_a_workers[i] as usize % COLUMNS] = 2;
            tile_max_heights[player_b_workers[i] as usize / COLUMNS][player_b_workers[i] as usize % COLUMNS] = 2;
        }

        let mut current_block_count = 0;
        while current_block_count < block_amount {
            loop {
                let tile = rng.gen_range(0..(ROWS * COLUMNS));
                if tile_heights[tile / COLUMNS][tile % COLUMNS] < tile_max_heights[tile / COLUMNS][tile % COLUMNS] {
                    tile_heights[tile / COLUMNS][tile % COLUMNS] += 1;
                    break;
                }
            }
            current_block_count += 1;
        }

        let player_a_turn = block_amount % 2 == 0;

        return GenericSantoriniGameState::new(player_a_workers, player_b_workers, tile_heights, player_a_turn)
            .expect("Randomly generated invalid game state, this should not be possible");
    }
}