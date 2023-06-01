use core::ops::{Index, IndexMut};

use glam::Vec2;

use crate::triangle::Triangle2;

pub struct Quad2(pub [Vec2; 4]);
impl Quad2 {
    pub fn contains(&self, pt: Vec2) -> bool {
        let [v1, v2, v3, v4] = self.0;
        Triangle2([v1, v2, v3]).contains(pt) || Triangle2([v3, v4, v1]).contains(pt)
    }
}

impl Index<usize> for Quad2 {
    type Output = Vec2;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Quad2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(feature = "macroquad-render")]
impl Quad2 {
    pub fn draw(&self, color: macroquad::prelude::Color) {
        let [v1, v2, v3, v4] = self.0;
        macroquad::shapes::draw_triangle(v1, v2, v3, color);
        macroquad::shapes::draw_triangle(v3, v4, v1, color);
    }
    pub fn draw_lines(&self, thickness: f32, color: macroquad::prelude::Color) {
        for i in 0..4 {
            let v1 = self[i];
            let v2 = self[(i + 1) & 3];
            macroquad::shapes::draw_line(v1.x, v1.y, v2.x, v2.y, thickness, color);
        }
    }
}
