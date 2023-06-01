use core::ops::{Index, IndexMut};

use glam::Vec2;

use crate::triangle::Triangle2;

pub struct Quad2d(pub [Vec2; 4]);
impl Quad2d {
    pub fn contains(&self, pt: Vec2) -> bool {
        let [v1, v2, v3, v4] = self.0;
        Triangle2([v1, v2, v3]).contains(pt) || Triangle2([v3, v4, v1]).contains(pt)
    }
}

impl Index<usize> for Quad2d {
    type Output = Vec2;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Quad2d {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
