use core::ops::Range;

pub struct BitSet2d {
    bits: Vec<u64>,
    x_range: Range<isize>,
    y_range: Range<isize>,
}
impl BitSet2d {
    pub fn new() -> Self {
        Self {
            bits: Vec::new(),
            x_range: 0..0,
            y_range: 0..0,
        }
    }

    pub fn insert(&mut self, (x, y): (isize, isize)) {
        self.cover((x, y));
        let (index, bit) = self.index((x, y));
        self.bits[index] |= 1 << bit;
    }

    pub fn remove(&mut self, (x, y): (isize, isize)) -> bool {
        if self.x_range.contains(&x) && self.y_range.contains(&y) {
            let (index, bit) = self.index((x, y));
            let bit = self.bits[index] & (1 << bit);
            self.bits[index] ^= bit;
            bit != 0
        } else {
            false
        }
    }

    pub fn contains(&self, (x, y): (isize, isize)) -> bool {
        if !self.x_range.contains(&x) || !self.y_range.contains(&y) {
            return false;
        }
        let (index, bit) = self.index((x, y));
        get_bit(&self.bits[index], bit)
    }

    fn cover(&mut self, (x, y): (isize, isize)) {
        if self.bits.is_empty() {
            self.x_range = x_chunk_cover(x);
            self.y_range = y..(y + 1);
            self.bits.push(0);
        } else {
            let Range {
                start: x_min,
                end: x_max,
            } = x_chunk_cover(x);
            let new_x_range = self.x_range.start.min(x_min)..self.x_range.end.max(x_max);
            let new_y_range = self.y_range.start.min(y)..self.y_range.end.max(y);

            if new_x_range != self.x_range || new_y_range != self.y_range {
                let new_x_chunks = (new_x_range.len() + 63) >> 6;
                self.bits.resize(new_x_chunks * new_y_range.len(), 0);

                let old_row_span = self.x_range.len() >> 6;
                let new_row_span = new_x_range.len() >> 6;
                let old_y_istart = (new_y_range.start..self.y_range.start).len();
                let old_y_iend = (new_y_range.start..self.y_range.end).len();

                self.bits[(old_y_iend * new_row_span)..].fill(0);
                for (old_iy, new_iy) in (0..self.y_range.len()).zip(old_y_istart..old_y_iend).rev()
                {
                    let old_x_istart = (new_x_range.start..self.x_range.start).len() >> 6;
                    let old_x_iend = (new_x_range.start..self.x_range.end).len() >> 6;

                    let new_idx = new_iy * new_row_span;
                    let old_idx = old_iy * old_row_span + old_x_istart;
                    let len = (old_x_istart..old_x_iend).len();

                    self.bits[(new_idx + old_x_iend)..(new_idx + new_row_span)].fill(0);
                    self.bits
                        .copy_within(old_idx..(old_idx + len), new_idx + old_x_istart);
                    self.bits[new_idx..(new_idx + old_x_istart)].fill(0);
                }
                self.bits[..(old_y_istart * new_row_span)].fill(0);
            }

            self.x_range = new_x_range;
            self.y_range = new_y_range;
        }
    }

    fn index(&self, (x, y): (isize, isize)) -> (usize, u32) {
        Self::index_impl((x, y), self.x_range.clone(), self.y_range.clone())
    }
    fn index_impl(
        (x, y): (isize, isize),
        x_range: Range<isize>,
        y_range: Range<isize>,
    ) -> (usize, u32) {
        // `x` needs to be rounded to avoid needing to move chunks.
        let row_span = x_range.len() >> 6;
        let x = x - x_range.start;
        let y = y - y_range.start;
        debug_assert!(x >= 0, "{x:} >= 0");
        debug_assert!(y >= 0, "{y:} >= 0");
        ((x as usize >> 6) + y as usize * row_span, (x & 63) as u32)
    }
}

fn x_chunk_cover(x: isize) -> Range<isize> {
    let min = if x >= 0 { x & !63 } else { (x | 63) - 63 };
    min..(min + 64)
}

fn get_bit(chunk: &u64, bit: u32) -> bool {
    ((*chunk >> bit) & 1) != 0
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rand::Rng;

    use super::*;

    #[test]
    fn test_x_range() {
        for Range { start, end } in [-128..-64, -64..0, 0..64, 64..128] {
            for x in start..end {
                assert_eq!(x_chunk_cover(x), start..end, "x={x:}");
            }
        }
    }

    #[test]
    fn test_insert() {
        let mut sut = BitSet2d::new();
        sut.insert((0, 2));
        assert_eq!(sut.x_range, 0..64);
        assert_eq!(sut.y_range, 2..3);
        assert_eq!(&sut.bits, &[1]);

        sut.insert((0, 0));
        assert_eq!(sut.x_range, 0..64);
        assert_eq!(sut.y_range, 0..3);
        assert_eq!(&sut.bits, &[1, 0, 1]);

        sut.insert((80, 0));
        assert_eq!(sut.x_range, 0..128);
        assert_eq!(sut.y_range, 0..3);
        assert_eq!(&sut.bits, &[1, 1 << (80 - 64), 0, 0, 1, 0]);

        sut.insert((80, 2));
        assert_eq!(sut.x_range, 0..128);
        assert_eq!(sut.y_range, 0..3);
        assert_eq!(&sut.bits, &[1, 1 << (80 - 64), 0, 0, 1, 1 << (80 - 64)]);
    }

    #[test]
    fn test_contains() {
        let mut sut = BitSet2d::new();
        sut.insert((0, 2));
        assert_eq!(sut.contains((0, 2)), true);
        assert_eq!(sut.contains((0, 0)), false);
        assert_eq!(sut.contains((80, 0)), false);
        assert_eq!(sut.contains((80, 2)), false);

        sut.insert((0, 0));
        assert_eq!(sut.contains((0, 2)), true);
        assert_eq!(sut.contains((0, 0)), true);
        assert_eq!(sut.contains((80, 0)), false);
        assert_eq!(sut.contains((80, 2)), false);

        sut.insert((80, 0));
        assert_eq!(sut.contains((0, 2)), true);
        assert_eq!(sut.contains((0, 0)), true);
        assert_eq!(sut.contains((80, 0)), true);
        assert_eq!(sut.contains((80, 2)), false);

        sut.insert((80, 2));
        assert_eq!(sut.contains((0, 2)), true);
        assert_eq!(sut.contains((0, 0)), true);
        assert_eq!(sut.contains((80, 0)), true);
        assert_eq!(sut.contains((80, 2)), true);
    }

    #[test]
    fn test_remove() {
        let mut sut = BitSet2d::new();
        sut.insert((0, 2));
        assert_eq!(sut.remove((0, 2)), true);
        assert_eq!(sut.remove((0, 2)), false);
    }

    #[ignore = "expensive fuzzing, should be covered by other tests"]
    #[test]
    fn test_insert_fuzzing() {
        let mut prng = rand::thread_rng();
        for _ in 0..1000 {
            for _ in 0..prng.gen_range(1..=100) {
                let mut expected = HashSet::<(isize, isize)>::new();
                let mut sut = BitSet2d::new();

                let p: (isize, isize) = (prng.gen_range(-60..=60), prng.gen_range(-60..=60));
                expected.insert(p);
                sut.insert(p);

                for iy in -60..=60 {
                    for ix in -60..=60 {
                        let p = (ix, iy);
                        assert_eq!(sut.contains(p), expected.contains(&p), "set={:?}", expected);
                    }
                }
            }
        }
    }
}
