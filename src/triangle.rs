use core::ops::{Index, IndexMut};

use glam::Vec2;

pub struct Triangle2(pub [Vec2; 3]);
impl Triangle2 {
    pub fn contains(&self, pt: Vec2) -> bool {
        let [v1, v2, v3] = self.0;

        let d1 = (pt - v2).perp_dot(v1 - v2);
        let d2 = (pt - v3).perp_dot(v2 - v3);
        let d3 = (pt - v1).perp_dot(v3 - v1);

        return d1.signum() == d2.signum() && d1.signum() == d3.signum();
    }
}

impl Index<usize> for Triangle2 {
    type Output = Vec2;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for Triangle2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(feature = "macroquad-render")]
impl Triangle2 {
    pub fn draw(&self, color: macroquad::prelude::Color) {
        let [v1, v2, v3] = self.0;
        macroquad::shapes::draw_triangle(v1, v2, v3, color);
    }
    pub fn draw_lines(&self, thickness: f32, color: macroquad::prelude::Color) {
        let [v1, v2, v3] = self.0;
        macroquad::shapes::draw_triangle_lines(v1, v2, v3, thickness, color);
    }
}
