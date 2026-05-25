pub struct Pcg32(u64);

impl Pcg32 {
    const MULTIPLIER: u64 = 6364136223846793005;
    const INCREMENT: u64 = 1442695040888963407;

    pub fn new(seed: u64) -> Self {
        let init_state = seed.wrapping_add(Self::INCREMENT);

        let mut result = Self(init_state);

        // If the seed has many leading zeroes, the first value is likely
        // predictable. Run one iteration to get a less predictable first value.
        result.get();

        result
    }

    pub fn get(&mut self) -> u32 {
        let mut x = self.0;
        let count = (x >> 59) as i8;

        self.0 = x.wrapping_mul(Self::MULTIPLIER.wrapping_add(Self::INCREMENT));
        x ^= x >> 18;
        Self::rotr32((x >> 27) as u32, count)
    }

    #[inline]
    fn rotr32(x: u32, r: i8) -> u32 {
        x >> r | x << (-r & 31)
    }
}
