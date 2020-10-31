use crate::lib_core::{
    math::{index_1d, index_3d, FixedNumber},
    time::GameFrame,
};

use super::{Chunk, Palette, PaletteIndex, Voxel};

pub struct ChunkManager {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    // The maximum allowed steps to calculate distance fields for
    max_distance_field: u8,
    pub chunk_size: (usize, usize, usize),
    pub voxel_resolution: FixedNumber,
    pub last_update: GameFrame,
    current_frame: GameFrame,
    pub chunks: Vec<Chunk>,
}

impl ChunkManager {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize) -> Self {
        let capacity = x_depth * y_depth * z_depth;

        let chunk_size = 16;
        let chunk_size = (chunk_size, chunk_size, chunk_size);

        let max_distance_field = 4;

        let voxel_resolution = FixedNumber::fraction(2.into());

        let mut d = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            d.push(0);
        }

        use rayon::prelude::*;

        let chunks = d
            .par_iter()
            .map(|_| Chunk::new(chunk_size.0, chunk_size.1, chunk_size.2))
            .collect();

        Self {
            voxel_resolution,
            max_distance_field,
            x_depth,
            y_depth,
            z_depth,
            last_update: 0,
            current_frame: 0,
            chunks,
            chunk_size,
        }
    }

    pub fn last_update(&self) -> GameFrame {
        self.last_update
    }

    pub fn update_frame(&mut self, frame: GameFrame) {
        self.current_frame = frame;
        for chunk in self.chunks.iter_mut() {
            chunk.update(frame);
        }
    }

    pub fn calculate_distance_fields(&mut self) {
        //TODO: calculate distances for all voxels.
        // Summed area tables?
    }

    pub fn capacity(&self) -> (usize, usize, usize) {
        (self.x_depth, self.y_depth, self.z_depth)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    fn index_1d(&self, x: usize, y: usize, z: usize) -> usize {
        index_1d(x, y, z, self.x_depth, self.y_depth, self.z_depth)
    }

    fn index_3d(&self, i: usize) -> (usize, usize, usize) {
        index_3d(i, self.x_depth, self.y_depth, self.z_depth)
    }
}
