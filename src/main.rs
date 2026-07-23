mod dda;

use crate::dda::DDA;
use raylib::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

const MAIN_COLOR: u32 = 0x96_00_FF_FF;
const RESOLUTION: usize = 128;
const SCALE: i32 = 4;
const CHUNK_SIZE: i32 = RESOLUTION as i32 * SCALE;

struct Chunk {
    pub key: i32,
    pub x: i32,
    pub y: i32,
    pub buffer: Box<[u8; 4 * RESOLUTION * RESOLUTION]>,
    pub texture: Texture2D,
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1024, 1024)
        .title("Hello, World")
        .undecorated()
        .resizable()
        .build();

    let mut pan = Vector2::new(0.0, 0.0);
    let mut last_mouse_x: i32 = -1;
    let mut last_mouse_y: i32 = -1;

    let mut chunks: HashMap<i32, Chunk> = HashMap::new();
    let mut chunks_to_update: HashSet<i32> = HashSet::new();

    while !rl.window_should_close() {
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            break;
        }

        let m_x = rl.get_mouse_x();
        let m_y = rl.get_mouse_y();

        let panned_m_x = m_x - pan.x as i32;
        let panned_m_y = m_y - pan.y as i32;

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE) {
            if last_mouse_x >= 0 && last_mouse_y >= 0 {
                pan.x += (m_x - last_mouse_x) as f32;
                pan.y += (m_y - last_mouse_y) as f32;
            }

            last_mouse_x = m_x;
            last_mouse_y = m_y;
        } else if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            chunks_to_update.insert(put_pixel_global(
                &mut rl,
                &thread,
                &mut chunks,
                panned_m_x,
                panned_m_y,
            ));

            if last_mouse_x >= 0 && last_mouse_y >= 0 {
                let dda = DDA::new(last_mouse_x, last_mouse_y, m_x, m_y);
                for position in dda {
                    chunks_to_update.insert(put_pixel_global(
                        &mut rl,
                        &thread,
                        &mut chunks,
                        position.0 - pan.x as i32,
                        position.1 - pan.y as i32,
                    ));
                }
            }

            last_mouse_x = m_x;
            last_mouse_y = m_y;

            for key in chunks_to_update.iter() {
                if let Some(chunk) = chunks.get_mut(&key) {
                    let buffer = chunk.buffer.deref();
                    chunk.texture.update_texture(buffer).ok();
                }
            }
            chunks_to_update.clear();
        } else {
            last_mouse_x = -1;
            last_mouse_y = -1;
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        for chunk in chunks.values_mut() {
            d.draw_texture_ex(
                &chunk.texture,
                Vector2::new(pan.x + chunk.x as f32, pan.y + chunk.y as f32),
                0.0,
                SCALE as f32,
                Color::get_color(MAIN_COLOR),
            );
        }

        d.draw_rectangle_lines(
            0,
            0,
            d.get_render_width(),
            d.get_render_height(),
            Color::get_color(MAIN_COLOR),
        );
    }

    fn put_pixel_global(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        chunks: &mut HashMap<i32, Chunk>,
        x: i32,
        y: i32,
    ) -> i32 {

        if let Some(chunk) = get_chunk_at(rl, &thread, chunks, x, y) {
            let buffer = chunk.buffer.deref_mut();
            put_pixel(buffer, x - chunk.x, y - chunk.y);
            return chunk.key;
        }

        -1
    }

    fn put_pixel(buffer: &mut [u8], x: i32, y: i32) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || x < 0 || y < 0 {
            return;
        }

        let index = canvas_to_index(x, y);
        buffer[index + 0] = u8::MAX;
        buffer[index + 1] = u8::MAX;
        buffer[index + 2] = u8::MAX;
        buffer[index + 3] = u8::MAX;
    }

    fn canvas_to_index(x: i32, y: i32) -> usize {
        ((x / SCALE + y / SCALE* (RESOLUTION as i32)) * 4) as usize
    }

    fn get_chunk_at<'a>(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        chunks: &'a mut HashMap<i32, Chunk>,
        x: i32,
        y: i32,
    ) -> Option<&'a mut Chunk> {
        let x = ((x as f32 / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32) as i32;
        let y = ((y as f32 / CHUNK_SIZE as f32).floor() * CHUNK_SIZE as f32) as i32;

        let key = x * 100000 + y;
        if chunks.contains_key(&key) {
            return chunks.get_mut(&key);
        } else {
            let i = Image::gen_image_color(RESOLUTION as i32, RESOLUTION as i32, Color::BLACK);
            if let Ok(texture) = rl.load_texture_from_image(thread, &i) {
                let c = Chunk {
                    key,
                    x,
                    y,
                    buffer: Box::new([0u8; RESOLUTION * RESOLUTION * 4]),
                    texture,
                };

                chunks.insert(key, c);
            }
        }

        None
    }
}
