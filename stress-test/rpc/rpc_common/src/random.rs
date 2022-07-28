/// taken from a java-version implementation:
/// http://developer.classpath.org/doc/java/util/Random-source.htmlv
#[allow(dead_code)]
pub struct FastRandom {
    seed: u64,
}

#[allow(dead_code)]
impl FastRandom {
    pub fn new(seed: u64) -> Self {
        Self {
            seed: Self::set_seed0(seed),
        }
    }

    pub fn get_next(&mut self) -> u64 {
        self.next(32).wrapping_shl(32).wrapping_add(self.next(32))
    }

    pub fn get_cur_seed(&self) -> u64 {
        self.seed
    }

    #[inline]
    fn next(&mut self, bits: usize) -> u64 {
        self.seed = (self
            .seed
            .wrapping_mul(0x5DEECE66D as u64)
            .wrapping_add(0xB as u64))
            & (((1 as u64) << 48) - 1);
        //        self.seed
        self.seed.wrapping_shr((48 - bits) as u32)
    }

    #[inline]
    fn set_seed0(seed: u64) -> u64 {
        (seed ^ (0x5DEECE66D as u64)) & (((1 as u64) << 48) - 1)
    }
}
