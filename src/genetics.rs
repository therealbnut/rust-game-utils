use bitvec::{macros::internal::funty::Integral, prelude::*};
use rand::Rng;

#[derive(Clone)]
pub struct DNA {
    genetic_info: BitVec,
}
impl DNA {
    const DATA_START: usize = 16;

    pub fn new(mutation_rate: u8, crossover_rate: u8) -> Self {
        let mut genetic_info = BitVec::new();
        genetic_info.resize(Self::DATA_START, false);
        genetic_info[0_..8].store(mutation_rate);
        genetic_info[8..16].store(crossover_rate);
        Self { genetic_info }
    }

    pub fn reproduce_asexual(that: &Self, rng: &mut impl Rng) -> Self {
        let mut new_dna = that.clone();
        new_dna.mutate(rng);
        new_dna
    }
    pub fn reproduce_sexual(lhs: &Self, rhs: &Self, rng: &mut impl Rng) -> Option<Self> {
        if lhs.genetic_info.len() != rhs.genetic_info.len() {
            return None;
        }
        let mut new_dna = rhs.clone();
        new_dna.crossover(&rhs, rng);
        new_dna.mutate(rng);
        Some(new_dna)
    }

    pub fn target_mutations(&self) -> usize {
        self.genetic_info[0..8].load_le::<usize>() + 1
    }
    pub fn target_crossovers(&self) -> usize {
        self.genetic_info[8..16].load_le::<usize>() + 1
    }

    pub fn crossover(&mut self, that: &Self, rng: &mut impl Rng) {
        let target_crossovers = self.target_crossovers();
        debug_assert_eq!(self.genetic_info.len(), that.genetic_info.len());
        let mut skip: bool = rng.gen();
        let mut prev_idx = 0;
        for idx in (self.rand_locations(target_crossovers, rng)).chain([self.genetic_info.len()]) {
            if !skip {
                self.genetic_info[prev_idx..idx]
                    .copy_from_bitslice(&that.genetic_info[prev_idx..idx]);
            }
            skip = !skip;
            prev_idx = idx;
        }
    }

    pub fn mutate<R: Rng>(&mut self, rng: &mut R) {
        let target_mutations = self.target_mutations();
        let mut iter = self.rand_locations(target_mutations, rng);
        while let Some(index) = iter.next() {
            let mut value = self.genetic_info[index] as u32;
            mutate::<R>(&mut value, u8::MAX as u32, iter.rng());
            self.genetic_info[index..][..1].store(iter.rng().gen::<bool>() as u32);
        }
    }

    pub fn reader(&self) -> DNAReader {
        DNAReader {
            slice: &self.genetic_info[Self::DATA_START..],
        }
    }

    #[inline]
    pub fn append<I: Integral>(&mut self, that: I) {
        self.append_compact(that, I::BITS);
    }
    pub fn append_compact<I: Integral>(&mut self, that: I, len: u32) {
        let len = len as usize;
        let index = self.genetic_info.len();
        self.genetic_info.resize(index + len, false);
        self.genetic_info[index..][..len].store_le(that);
    }

    fn rand_locations<'r, R: Rng>(
        &self,
        target_count: usize,
        rng: &'r mut R,
    ) -> RandLocations<'r, R> {
        let len = self.genetic_info.len();
        RandLocations::new(target_count, len, rng)
    }
}

pub struct DNAReader<'a> {
    slice: &'a BitSlice,
}
impl<'a> DNAReader<'a> {
    #[inline]
    pub fn next<I: Integral>(&mut self) -> Option<I> {
        self.next_compact(I::BITS)
    }
    pub fn next_compact<I: Integral>(&mut self, len: u32) -> Option<I> {
        Some(self.next_slice(len)?.load_le())
    }
    fn next_slice(&mut self, len: u32) -> Option<&'a BitSlice> {
        let len = len as usize;
        if self.slice.len() < len {
            return None;
        }
        let slice = &self.slice[..len];
        self.slice = &self.slice[len..];
        Some(slice)
    }
}

struct RandLocations<'a, R: Rng> {
    rate: usize,
    index: usize,
    len: usize,
    rng: &'a mut R,
}
impl<'a, R: Rng> RandLocations<'a, R> {
    pub fn new(target_count: usize, len: usize, rng: &'a mut R) -> Self {
        debug_assert_ne!(target_count, 0);
        let rate = len / target_count;
        Self {
            rate,
            index: 0,
            len,
            rng,
        }
    }
    pub fn rng(&mut self) -> &mut R {
        self.rng
    }
}
impl<R: Rng> core::iter::Iterator for RandLocations<'_, R> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        self.index += self.rng.gen_range(0..=self.rate);
        if self.index >= self.len {
            return None;
        }
        let index = self.index;
        self.index += 1;
        Some(index)
    }
}

fn mutate<R: Rng>(value: &mut u32, max: u32, rng: &mut R) {
    *value = average(*value, rng.gen_range(0..=max)) as u32;
}
fn average(lhs: u32, rhs: u32) -> u32 {
    (((lhs as u64) + (rhs as u64)) >> 1) as u32
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    const TARGET_SIZE: usize = 1;
    const TARGET: [u32; 1] = [1234];
    const POP_SIZE: usize = 32;
    fn fitness_function(that: &DNA) -> u32 {
        let array = read_array(that);
        let mut count = 0;
        for i in 0..TARGET.len() {
            count = count.saturating_add((array[i] ^ TARGET[i]).count_ones());
        }
        count
    }
    fn read_array(dna: &DNA) -> [u32; TARGET_SIZE] {
        let mut iter = dna.reader();
        TARGET.map(|_| iter.next().unwrap())
    }

    #[test]
    fn test_asexual_evolve_value() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);

        let mut dna = DNA::new(6, 0);
        for _ in 0..TARGET.len() {
            dna.append(0u32);
        }
        let mut population = [0; POP_SIZE].map(|_| dna.clone());
        for gen in 1..=1000 {
            population.sort_by_key(fitness_function);
            if read_array(&population[0]) == TARGET {
                assert_eq!(gen, 171);
                break;
            }
            for i in (POP_SIZE / 2)..POP_SIZE {
                let parent = &population[rng.gen_range(0..POP_SIZE)];
                population[i] = DNA::reproduce_asexual(parent, &mut rng);
            }
        }

        assert_eq!(read_array(&population[0]), TARGET);
    }

    #[test]
    fn test_sexual_evolve_value() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);

        let mut dna = DNA::new(8, 128);
        for _ in 0..TARGET.len() {
            dna.append(0u32);
        }
        let mut population = [0; POP_SIZE].map(|_| dna.clone());
        for gen in 1..=1000 {
            population.sort_by_key(fitness_function);
            if read_array(&population[0]) == TARGET {
                assert_eq!(gen, 177);
                break;
            }
            for i in (POP_SIZE / 2)..POP_SIZE {
                let lhs = &population[rng.gen_range(0..(POP_SIZE / 2))];
                let rhs = &population[rng.gen_range(0..(POP_SIZE / 2))];
                population[i] = DNA::reproduce_sexual(lhs, rhs, &mut rng).unwrap();
            }
        }
        assert_eq!(read_array(&population[0]), TARGET);
    }
}
