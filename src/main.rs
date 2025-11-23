use std::time::{Duration, Instant};

use enum_iterator::{first, last, next, previous};
use macroquad::prelude::*;

use falling_sand::{Block, World};

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Falling Sand"),
        window_width: 600,
        window_height: 600,
        window_resizable: true,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new(30, 30);

    let mut timer = Instant::now();
    let delay = Duration::from_millis(250);

    let mut current_block = Block::Sand;

    loop {
        clear_background(BLACK);

        let (m_x, m_y) = mouse_position();
        let (grid_x, grid_y) = {
            let (s_x, s_y) = (m_x / 20.0, m_y / 20.0);

            (s_x.floor() as usize, s_y.floor() as usize)
        };

        if is_mouse_button_down(MouseButton::Left) {
            world.set_block(grid_x, grid_y, current_block);
        } else if is_mouse_button_down(MouseButton::Right) {
            world.set_block(grid_x, grid_y, Block::Air);
        }

        if mouse_wheel().1 > 0.0 {
            current_block = if let Some(block) = next(&current_block) {
                block
            } else {
                // wrap around to start
                first::<Block>().unwrap()
            };
        } else if mouse_wheel().1 < 0.0 {
            current_block = if let Some(block) = previous(&current_block) {
                block
            } else {
                // wrap around to end
                last::<Block>().unwrap()
            };
        }

        // keeps the game from running too fast
        if timer.elapsed() >= delay {
            world.update();
            timer = Instant::now();
        }

        // Draw world state
        for y in 0..world.get_height() {
            for x in 0..world.get_width() {
                if let Some(block) = world.get_block(x, y) {
                    if block == Block::Air {
                        continue;
                    }

                    let color = block.get_color();
                    draw_rectangle(x as f32 * 20.0, y as f32 * 20.0, 20.0, 20.0, color);
                }
            }
        }

        draw_text(format!("{current_block:?}").as_str(), 10.0, 60.0, 50.0, WHITE);

        next_frame().await
    }
}
