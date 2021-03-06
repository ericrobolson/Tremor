use crate::lib_core::{math::index_1d, math::index_3d, math::FixedNumber, time::GameFrame};

use super::{Voxel, VoxelNumeric};

pub struct Chunk {
    x_depth: usize,
    y_depth: usize,
    z_depth: usize,
    last_update: GameFrame,
    current_frame: GameFrame,
    voxels: Vec<u8>,
    mass: FixedNumber,
    inv_mass: FixedNumber,
    restitution: FixedNumber,
    active_voxels: usize,
    total_mass: i32,
    total_restitution: i32,
}

impl Chunk {
    pub fn new(x_depth: usize, y_depth: usize, z_depth: usize, distance_field: u8) -> Self {
        let capacity = x_depth * y_depth * z_depth;
        let mut voxels: Vec<u8> = Vec::with_capacity(capacity);

        let mut distance_field = distance_field;
        if distance_field < 1 {
            distance_field = 1;
        }

        // Always assign a voxel
        for _ in 0..capacity {
            let mut voxel: u8 = Voxel::Empty.into();
            voxel.set_distance_field(distance_field);
            voxels.push(voxel);
        }

        Self {
            x_depth,
            y_depth,
            z_depth,
            voxels,
            last_update: 0,
            current_frame: 0,
            mass: 0.into(),
            inv_mass: 0.into(),
            restitution: 0.into(),
            active_voxels: 0,
            total_mass: 0,
            total_restitution: 0,
        }
    }

    pub fn last_update(&self) -> GameFrame {
        self.last_update
    }

    pub fn update(&mut self, frame: GameFrame) {
        self.current_frame = frame;
    }

    pub fn capacity(&self) -> (usize, usize, usize) {
        (self.x_depth, self.y_depth, self.z_depth)
    }

    fn index_1d(&self, x: usize, y: usize, z: usize) -> usize {
        index_1d(x, y, z, self.x_depth, self.y_depth, self.z_depth)
    }

    fn index_3d(&self, i: usize) -> (usize, usize, usize) {
        index_3d(i, self.x_depth, self.y_depth, self.z_depth)
    }

    fn safe_wrap(&self, x: usize, y: usize, z: usize) -> (usize, usize, usize) {
        (x % self.x_depth, y % self.y_depth, z % self.z_depth)
    }

    pub fn voxels(&self) -> &Vec<u8> {
        &self.voxels
    }

    pub fn mass(&self) -> FixedNumber {
        self.mass
    }

    fn calculate_inv_mass(&self) -> FixedNumber {
        let mass = self.mass();
        if mass == 0.into() {
            return FixedNumber::MAX(); // Really large number
        }

        let inv_mass = FixedNumber::fraction(mass.into());

        if inv_mass == 0.into() {
            return FixedNumber::decimal_resolution_value();
        }

        return inv_mass;
    }

    pub fn inv_mass(&self) -> FixedNumber {
        self.inv_mass
    }

    pub fn restitution(&self) -> FixedNumber {
        self.restitution
    }

    pub fn voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        return self.voxels[self.index_1d(x, y, z)].voxel();
    }

    pub fn raw_voxel(&self, x: usize, y: usize, z: usize) -> u8 {
        self.voxels[self.index_1d(x, y, z)]
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let i = self.index_1d(x, y, z);

        // Set new physical properties
        {
            let prev_voxel = self.voxels[i].voxel();

            // Set mass to new mass
            self.total_mass -= prev_voxel.mass() as i32;
            self.total_mass += voxel.mass() as i32;

            self.mass = {
                let total_mass = self.total_mass / 100; // NOTE: need to scale this down or it explodes.
                let total_mass: FixedNumber = total_mass.into();
                total_mass / 100.into() // NOTE: need to scale this down or it explodes.
            };
            self.inv_mass = self.calculate_inv_mass();

            // Restitution
            self.total_restitution -= prev_voxel.restitution() as i32;
            self.total_restitution += voxel.restitution() as i32;
            self.restitution = {
                let total_restitution = self.total_restitution / 1000; // NOTE: need to scale this down
                let total_restitution: FixedNumber = total_restitution.into();
                total_restitution / 100.into() // NOTE: need to scale this down
            };

            if prev_voxel == Voxel::Empty && voxel != Voxel::Empty {
                self.active_voxels += 1;
            } else if prev_voxel != Voxel::Empty && voxel == Voxel::Empty {
                self.active_voxels -= 1;
            }
        }

        // Update the voxel
        self.voxels[i].set_voxel(voxel);
        self.last_update = self.current_frame;
    }

    pub fn set_distance_field(&mut self, x: usize, y: usize, z: usize, dist: u8) {
        let i = self.index_1d(x, y, z);
        self.voxels[i].set_distance_field(dist);
        self.last_update = self.current_frame;
    }

    pub fn occluded(&self, x: usize, y: usize, z: usize) -> bool {
        let (x, y, z) = self.safe_wrap(x, y, z);
        // If on an edge, return false
        if x == 0
            || y == 0
            || z == 0
            || (x == self.x_depth - 1)
            || (y == self.y_depth - 1)
            || (z == self.z_depth - 1)
        {
            return false;
        }

        // Check all neighbors
        if self.voxel(x - 1, y, z) == Voxel::Empty {
            return false;
        }

        if self.voxel(x + 1, y, z) == Voxel::Empty {
            return false;
        }

        if self.voxel(x, y - 1, z) == Voxel::Empty {
            return false;
        }

        if self.voxel(x, y + 1, z) == Voxel::Empty {
            return false;
        }

        if self.voxel(x, y, z - 1) == Voxel::Empty {
            return false;
        }

        if self.voxel(x, y, z + 1) == Voxel::Empty {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn VoxelChunk_New_DefaultsToEmpty() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = Chunk::new(x_depth, y_depth, z_depth, 1);

        assert_eq!(x_depth, chunk.x_depth);
        assert_eq!(y_depth, chunk.y_depth);
        assert_eq!(z_depth, chunk.z_depth);

        assert_eq!(x_depth * y_depth * z_depth, chunk.voxels.len());

        for voxel in chunk.voxels {
            assert_eq!(voxel.voxel(), Voxel::Empty);
        }
    }

    #[test]
    fn VoxelChunk_index_1d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = Chunk::new(x_depth, y_depth, z_depth, 1);

        let (x, y, z) = (0, 0, 0);
        let expected = 0;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);

        let (x, y, z) = (1, 2, 3);
        let expected = x + y * x_depth + z * x_depth * y_depth;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);

        // Boundary check
        let (x, y, z) = (x_depth, y_depth, z_depth);
        let expected = 0;
        let actual = chunk.index_1d(x, y, z);
        assert_eq!(expected, actual);
    }

    #[test]
    fn VoxelChunk_index_3d_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let chunk = Chunk::new(x_depth, y_depth, z_depth, 1);

        let (x, y, z) = (1, 2, 3);
        let expected = (x, y, z);
        let i = chunk.index_1d(x, y, z);
        let actual = chunk.index_3d(i);
        assert_eq!(expected, actual);

        let (x, y, z) = (3, 3, 2);
        let expected = (0, y, z);
        let i = chunk.index_1d(x, y, z);
        let actual = chunk.index_3d(i);
        assert_eq!(expected, actual);
    }

    #[test]
    fn VoxelChunk_set_voxels_works_as_expected() {
        let x_depth = 3;
        let y_depth = 4;
        let z_depth = 5;

        let mut chunk = Chunk::new(x_depth, y_depth, z_depth, 1);

        let voxel = Voxel::Cloth;
        let (x, y, z) = (0, 0, 0);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (0, 0, 0);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::Bone;
        let (x, y, z) = (2, 3, 1);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));

        let voxel = Voxel::DebugCollisionShape;
        let (x, y, z) = (3, 2, 4);
        chunk.set_voxel(x, y, z, voxel);
        assert_eq!(voxel, chunk.voxel(x, y, z));
    }
}
