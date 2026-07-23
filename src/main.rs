mod dda;

use crate::dda::DDA;
use raylib::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

const MAIN_COLOR: u32 = 0x96_00_FF_FF;
const CHUNK_RESOLUTION: usize = 128;
const CHUNK_SIZE: i32 = CHUNK_RESOLUTION as i32 * SCALE;
const SCALE: i32 = 4;
const GRID_INTERVAL: i32 = CHUNK_RESOLUTION as i32 / 4;

struct Chunk {
    pub key: i32,
    pub x: i32,
    pub y: i32,
    pub buffer: Box<[u8; 4 * CHUNK_RESOLUTION * CHUNK_RESOLUTION]>,
    pub texture: Texture2D,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1024, 1024)
        .title(env!("CARGO_PKG_NAME"))
        .undecorated()
        .resizable()
        .build();

    let mut pan = Vector2::new(0.0, 0.0);
    let mut last_m_x: i32 = -1;
    let mut last_m_y: i32 = -1;
    let mut has_last = false;

    let mut grid_opacity = 0.0;

    let mut chunks: HashMap<i32, Chunk> = HashMap::new();
    let mut chunks_to_update: HashSet<i32> = HashSet::new();

    while !rl.window_should_close() {
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            break;
        }

        let do_snap = rl.is_key_down(KeyboardKey::KEY_G);

        let t = rl.get_time() as f32;
        let dt = rl.get_frame_time();
        let w = rl.get_render_width();
        let h = rl.get_render_height();

        let m_x = rl.get_mouse_x();
        let m_y = rl.get_mouse_y();

        let left = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        let right = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);

        let c_snap = |x: i32| -> i32 {
            if do_snap {
                isnap_round(x, GRID_INTERVAL)
            } else {
                x
            }
        };

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE) {
            if has_last {
                pan.x += (m_x - last_m_x) as f32;
                pan.y += (m_y - last_m_y) as f32;
            }

            last_m_x = m_x;
            last_m_y = m_y;
            has_last = true;
        } else if left || right {
            let value = if left { u8::MAX } else { 0 };

            chunks_to_update.insert(put_pixel_global(
                &mut rl,
                &thread,
                &mut chunks,
                c_snap(m_x - pan.x as i32),
                c_snap(m_y - pan.y as i32),
                value,
            ));

            if has_last {
                let dda = DDA::new(
                    c_snap(last_m_x - pan.x as i32),
                    c_snap(last_m_y - pan.y as i32),
                    c_snap(m_x - pan.x as i32),
                    c_snap(m_y - pan.y as i32),
                );
                for position in dda {
                    chunks_to_update.insert(put_pixel_global(
                        &mut rl,
                        &thread,
                        &mut chunks,
                        position.0,
                        position.1,
                        value,
                    ));
                }
            }

            last_m_x = m_x;
            last_m_y = m_y;
            has_last = true;

            for key in chunks_to_update.iter() {
                if let Some(chunk) = chunks.get_mut(&key) {
                    let buffer = chunk.buffer.deref();
                    chunk.texture.update_texture(buffer).ok();
                }
            }
            chunks_to_update.clear();
        } else {
            has_last = false;
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        for chunk in chunks.values_mut() {
            let screen_pos = Vector2::new(pan.x + chunk.x as f32, pan.y + chunk.y as f32);

            if screen_pos.x > -CHUNK_SIZE as f32
                && screen_pos.y > -CHUNK_SIZE as f32
                && screen_pos.x < w as f32
                && screen_pos.y < h as f32
            {
                d.draw_texture_ex(
                    &chunk.texture,
                    screen_pos,
                    0.0,
                    SCALE as f32,
                    Color::get_color(MAIN_COLOR),
                );
            }
        }

        // draw grid points
        if do_snap {
            grid_opacity = lerp(grid_opacity, 0.8, 1.0 - 1e-5_f32.powf(dt));
        } else {
            grid_opacity *= 1e-1_f32.powf(dt);
        }

        if t < 2.0 || grid_opacity > 0.0 {
            let max_y = h / GRID_INTERVAL + 1;
            let t = (t - 0.4).max(0.0);

            if t > 0.0 {
                for x in 0..w / GRID_INTERVAL + 1 {
                    for y in 0..max_y {
                        let f = {
                            let mut fraction =
                                (y as f32 - t * h as f32 * 0.15) / max_y as f32 + 1.0;
                            if fraction > 1.0 {
                                fraction = 0.0;
                            }

                            fraction.clamp(0.0, 1.0).max(grid_opacity)
                        };
                        let c = Color::get_color(MAIN_COLOR).alpha(f);
                        d.draw_pixel(
                            ((x * GRID_INTERVAL) as f32 + (pan.x % GRID_INTERVAL as f32)) as i32,
                            ((y * GRID_INTERVAL) as f32 + (pan.y % GRID_INTERVAL as f32)) as i32,
                            c,
                        );
                    }
                }
            }
        }

        d.draw_rectangle_lines(0, 0, w, h, Color::get_color(MAIN_COLOR));
        if t < 1.0 {
            // we flash another one because it looks kind of cool
            let f = 1.0 - t;
            let c = Color::get_color(MAIN_COLOR).alpha(f);
            d.draw_rectangle_lines(16, 16, w - 32, h - 32, c);
            d.draw_rectangle(0, 0, w, h, c.alpha(f * f * f * f * 0.5));
        }
    }

    fn put_pixel_global(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        chunks: &mut HashMap<i32, Chunk>,
        x: i32,
        y: i32,
        value: u8,
    ) -> i32 {
        if let Some(chunk) = get_chunk_at(rl, &thread, chunks, x, y) {
            let buffer = chunk.buffer.deref_mut();
            put_pixel(buffer, x - chunk.x, y - chunk.y, value);
            return chunk.key;
        }

        -1
    }

    fn put_pixel(buffer: &mut [u8], x: i32, y: i32, value: u8) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || x < 0 || y < 0 {
            return;
        }

        let index = canvas_to_index(x, y);
        // RGBA
        buffer[index + 0] = value;
        buffer[index + 1] = value;
        buffer[index + 2] = value;
        buffer[index + 3] = u8::MAX;
    }

    fn canvas_to_index(x: i32, y: i32) -> usize {
        ((x / SCALE + y / SCALE * (CHUNK_RESOLUTION as i32)) * 4) as usize
    }

    fn get_chunk_at<'a>(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        chunks: &'a mut HashMap<i32, Chunk>,
        x: i32,
        y: i32,
    ) -> Option<&'a mut Chunk> {
        let x = isnap(x, CHUNK_SIZE);
        let y = isnap(y, CHUNK_SIZE);

        let key = x * 100000 + y;
        if chunks.contains_key(&key) {
            return chunks.get_mut(&key);
        } else {
            let i = Image::gen_image_color(
                CHUNK_RESOLUTION as i32,
                CHUNK_RESOLUTION as i32,
                Color::BLACK,
            );
            if let Ok(texture) = rl.load_texture_from_image(thread, &i) {
                let c = Chunk {
                    key,
                    x,
                    y,
                    buffer: Box::new([0u8; CHUNK_RESOLUTION * CHUNK_RESOLUTION * 4]),
                    texture,
                };

                chunks.insert(key, c);
            }
        }

        None
    }
}

fn snap(value: f32, interval: f32) -> f32 {
    (value / interval).floor() * interval
}

fn isnap(value: i32, interval: i32) -> i32 {
    ((value as f32 / interval as f32).floor() * interval as f32) as i32
}

fn snap_round(value: f32, interval: f32) -> f32 {
    (value / interval).round() * interval
}

fn isnap_round(value: i32, interval: i32) -> i32 {
    ((value as f32 / interval as f32).round() * interval as f32) as i32
}
