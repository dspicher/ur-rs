use bitcoin_hashes::Hash;
use rand_xoshiro::rand_core::RngCore;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;

#[allow(clippy::module_name_repetitions)]
pub struct Xoshiro256 {
    inner: Xoshiro256StarStar,
}

impl From<Xoshiro256StarStar> for Xoshiro256 {
    fn from(from: Xoshiro256StarStar) -> Self {
        Self { inner: from }
    }
}

impl From<&[u8]> for Xoshiro256 {
    fn from(from: &[u8]) -> Self {
        let hash = bitcoin_hashes::sha256::Hash::hash(from);
        Self::from(hash.into_inner())
    }
}

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
impl Xoshiro256 {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.inner.next_u64()
    }

    pub fn next_double(&mut self) -> f64 {
        self.next() as f64 / (u64::MAX as f64 + 1.0)
    }

    pub fn next_int(&mut self, low: u64, high: u64) -> u64 {
        (self.next_double() * ((high - low + 1) as f64)) as u64 + low
    }

    pub fn next_byte(&mut self) -> u8 {
        self.next_int(0, 255) as u8
    }

    pub fn next_bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.next_byte()).collect()
    }

    #[must_use]
    pub fn from_crc(bytes: &[u8]) -> Self {
        Self::from(&crate::crc32().checksum(bytes).to_be_bytes()[..])
    }

    pub fn shuffled<T>(&mut self, mut items: Vec<T>) -> Vec<T> {
        let mut shuffled = Vec::<T>::with_capacity(items.len());
        while !items.is_empty() {
            let index = self.next_int(0, (items.len() - 1) as u64) as usize;
            let item = items.remove(index);
            shuffled.push(item);
        }
        shuffled
    }

    pub fn choose_degree(&mut self, length: usize) -> anyhow::Result<u32> {
        let degree_weights: Vec<f64> = (1..=length).map(|x| 1.0 / x as f64).collect();
        let mut sampler = crate::sampler::Weighted::new(degree_weights)?;
        Ok(sampler.next(self)? + 1)
    }
}

impl From<&str> for Xoshiro256 {
    fn from(value: &str) -> Self {
        let hash = bitcoin_hashes::sha256::Hash::hash(value.as_bytes());
        Self::from(hash.into_inner())
    }
}

impl From<[u8; 32]> for Xoshiro256 {
    fn from(value: [u8; 32]) -> Self {
        let mut s = [0_u8; 32];
        for i in 0..4 {
            let mut v: u64 = 0;
            for n in 0..8 {
                v <<= 8;
                v |= u64::from(*value.get(8 * i + n).unwrap());
            }
            let bytes = v.to_le_bytes();
            for n in 0..8 {
                *s.get_mut(8 * i + n).unwrap() = *bytes.get(n).unwrap();
            }
        }
        Xoshiro256StarStar::from_seed(s).into()
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    #[must_use]
    pub fn make_message(seed: &str, size: usize) -> Vec<u8> {
        let mut xoshiro = Xoshiro256::from(seed);
        xoshiro.next_bytes(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_1() {
        let mut rng = Xoshiro256::from("Wolf");
        let expected = vec![
            42, 81, 85, 8, 82, 84, 76, 73, 70, 88, 2, 74, 40, 48, 77, 54, 88, 7, 5, 88, 37, 25, 82,
            13, 69, 59, 30, 39, 11, 82, 19, 99, 45, 87, 30, 15, 32, 22, 89, 44, 92, 77, 29, 78, 4,
            92, 44, 68, 92, 69, 1, 42, 89, 50, 37, 84, 63, 34, 32, 3, 17, 62, 40, 98, 82, 89, 24,
            43, 85, 39, 15, 3, 99, 29, 20, 42, 27, 10, 85, 66, 50, 35, 69, 70, 70, 74, 30, 13, 72,
            54, 11, 5, 70, 55, 91, 52, 10, 43, 43, 52,
        ];
        for e in expected {
            assert_eq!(rng.next() % 100, e);
        }
    }

    #[test]
    fn test_rng_2() {
        let mut rng = Xoshiro256::from_crc(b"Wolf");
        let expected = vec![
            88, 44, 94, 74, 0, 99, 7, 77, 68, 35, 47, 78, 19, 21, 50, 15, 42, 36, 91, 11, 85, 39,
            64, 22, 57, 11, 25, 12, 1, 91, 17, 75, 29, 47, 88, 11, 68, 58, 27, 65, 21, 54, 47, 54,
            73, 83, 23, 58, 75, 27, 26, 15, 60, 36, 30, 21, 55, 57, 77, 76, 75, 47, 53, 76, 9, 91,
            14, 69, 3, 95, 11, 73, 20, 99, 68, 61, 3, 98, 36, 98, 56, 65, 14, 80, 74, 57, 63, 68,
            51, 56, 24, 39, 53, 80, 57, 51, 81, 3, 1, 30,
        ];
        for e in expected {
            assert_eq!(rng.next() % 100, e);
        }
    }

    #[test]
    fn test_rng_3() {
        let mut rng = Xoshiro256::from("Wolf");
        let expected = vec![
            6, 5, 8, 4, 10, 5, 7, 10, 4, 9, 10, 9, 7, 7, 1, 1, 2, 9, 9, 2, 6, 4, 5, 7, 8, 5, 4, 2,
            3, 8, 7, 4, 5, 1, 10, 9, 3, 10, 2, 6, 8, 5, 7, 9, 3, 1, 5, 2, 7, 1, 4, 4, 4, 4, 9, 4,
            5, 5, 6, 9, 5, 1, 2, 8, 3, 3, 2, 8, 4, 3, 2, 1, 10, 8, 9, 3, 10, 8, 5, 5, 6, 7, 10, 5,
            8, 9, 4, 6, 4, 2, 10, 2, 1, 7, 9, 6, 7, 4, 2, 5,
        ];
        for e in expected {
            assert_eq!(rng.next_int(1, 10), e);
        }
    }

    #[test]
    fn test_shuffle() {
        let mut rng = Xoshiro256::from("Wolf");
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let expected = vec![
            vec![6, 4, 9, 3, 10, 5, 7, 8, 1, 2],
            vec![10, 8, 6, 5, 1, 2, 3, 9, 7, 4],
            vec![6, 4, 5, 8, 9, 3, 2, 1, 7, 10],
            vec![7, 3, 5, 1, 10, 9, 4, 8, 2, 6],
            vec![8, 5, 7, 10, 2, 1, 4, 3, 9, 6],
            vec![4, 3, 5, 6, 10, 2, 7, 8, 9, 1],
            vec![5, 1, 3, 9, 4, 6, 2, 10, 7, 8],
            vec![2, 1, 10, 8, 9, 4, 7, 6, 3, 5],
            vec![6, 7, 10, 4, 8, 9, 2, 3, 1, 5],
            vec![10, 2, 1, 7, 9, 5, 6, 3, 4, 8],
        ];
        for e in expected {
            let shuffled = rng.shuffled(values.clone());
            assert_eq!(shuffled, e);
        }
    }
}
