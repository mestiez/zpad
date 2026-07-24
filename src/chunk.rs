use crate::{BUFFER_SIZE, CHUNK_SIZE};
use raylib::prelude::Texture2D;
use raylib::texture::RaylibTexture2D;
use std::ops::{Deref, DerefMut};

pub struct Chunk {
    pub c_x: i32,
    pub c_y: i32,
    pub buffer: Box<[u8; BUFFER_SIZE]>,
    pub texture: Texture2D,
}

impl Chunk {
    pub fn x(&self) -> i32 {
        self.c_x * CHUNK_SIZE
    }

    pub fn y(&self) -> i32 {
        self.c_y * CHUNK_SIZE
    }

    pub fn update_texture(&mut self) {
        self.texture.update_texture(self.buffer.deref()).ok();
    }

    pub fn get_buffer(&self) -> &[u8; BUFFER_SIZE] {
        self.buffer.deref()
    }

    pub fn get_buffer_mut(&mut self) -> &mut [u8; BUFFER_SIZE] {
        self.buffer.deref_mut()
    }
}
