use crate::random::FastRandom;

pub struct Payload<const N: usize> {
    pub checksum: u64,
    pub arr: [u8; N],
}

impl<const N: usize> Payload<N> {

    pub fn create(random_seed: u64) -> Self {
        let mut arr: [u8; N] = [0 as u8; N];
        let mut random = FastRandom::new(random_seed);
        for i in 0..N {
            let r = random.get_next() as u8;
            arr[i] = r;
        }
        let mut res = Self {
            checksum: 0,
            arr: arr,
        };
        res.checksum = res.calculate_checksum();
        res
    }

    pub fn checksum_ok(&self) -> bool {
        self.calculate_checksum() == self.checksum
    }

    fn calculate_checksum(&self) -> u64 {
        use core::hash::BuildHasher;
        use hashbrown::hash_map::DefaultHashBuilder;
        use core::hash::{Hash, Hasher};
        let mut s = DefaultHashBuilder::with_seed(0).build_hasher();
        self.arr.hash(&mut s);
        s.finish()
    }
}
