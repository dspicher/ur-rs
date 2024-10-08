extern crate alloc;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Weighted {
    aliases: Vec<u32>,
    probs: Vec<f64>,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
impl Weighted {
    pub fn new(mut weights: Vec<f64>) -> Self {
        assert!(
            !weights.iter().any(|&p| p < 0.0),
            "negative probability encountered"
        );
        let summed = weights.iter().sum::<f64>();
        assert!(summed > 0.0, "probabilities don't sum to a positive value");
        let count = weights.len();
        for w in &mut weights {
            *w *= count as f64 / summed;
        }
        let (mut s, mut l): (Vec<usize>, Vec<usize>) = (1..=count)
            .map(|j| count - j)
            .partition(|&j| weights[j] < 1.0);

        let mut probs: Vec<f64> = alloc::vec![0.0; count];
        let mut aliases: Vec<u32> = alloc::vec![0; count];

        while !s.is_empty() && !l.is_empty() {
            let a = s.remove(s.len() - 1);
            let g = l.remove(l.len() - 1);
            probs[a] = weights[a];
            aliases[a] = g as u32;
            weights[g] += weights[a] - 1.0;
            if weights[g] < 1.0 {
                s.push(g);
            } else {
                l.push(g);
            }
        }

        while !l.is_empty() {
            let g = l.remove(l.len() - 1);
            probs[g] = 1.0;
        }

        while !s.is_empty() {
            let a = s.remove(s.len() - 1);
            probs[a] = 1.0;
        }

        Self { aliases, probs }
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn next(&self, xoshiro: &mut crate::xoshiro::Xoshiro256) -> u32 {
        let r1 = xoshiro.next_double();
        let r2 = xoshiro.next_double();
        let n = self.probs.len();
        let i = (n as f64 * r1) as usize;
        if r2 < self.probs[i] {
            i as u32
        } else {
            self.aliases[i]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler() {
        let weights = vec![1.0, 2.0, 4.0, 8.0];
        let mut xoshiro = crate::xoshiro::Xoshiro256::from("Wolf");
        let sampler = Weighted::new(weights);

        let expected_samples = vec![
            3, 3, 3, 3, 3, 3, 3, 0, 2, 3, 3, 3, 3, 1, 2, 2, 1, 3, 3, 2, 3, 3, 1, 1, 2, 1, 1, 3, 1,
            3, 1, 2, 0, 2, 1, 0, 3, 3, 3, 1, 3, 3, 3, 3, 1, 3, 2, 3, 2, 2, 3, 3, 3, 3, 2, 3, 3, 0,
            3, 3, 3, 3, 1, 2, 3, 3, 2, 2, 2, 1, 2, 2, 1, 2, 3, 1, 3, 0, 3, 2, 3, 3, 3, 3, 3, 3, 3,
            3, 2, 3, 1, 3, 3, 2, 0, 2, 2, 3, 1, 1, 2, 3, 2, 3, 3, 3, 3, 2, 3, 3, 3, 3, 3, 2, 3, 1,
            2, 1, 1, 3, 1, 3, 2, 2, 3, 3, 3, 1, 3, 3, 3, 3, 3, 3, 3, 3, 2, 3, 2, 3, 3, 1, 2, 3, 3,
            1, 3, 2, 3, 3, 3, 2, 3, 1, 3, 0, 3, 2, 1, 1, 3, 1, 3, 2, 3, 3, 3, 3, 2, 0, 3, 3, 1, 3,
            0, 2, 1, 3, 3, 1, 1, 3, 1, 2, 3, 3, 3, 0, 2, 3, 2, 0, 1, 3, 3, 3, 2, 2, 2, 3, 3, 3, 3,
            3, 2, 3, 3, 3, 3, 2, 3, 3, 2, 0, 2, 3, 3, 3, 3, 2, 1, 1, 1, 2, 1, 3, 3, 3, 2, 2, 3, 3,
            1, 2, 3, 0, 3, 2, 3, 3, 3, 3, 0, 2, 2, 3, 2, 2, 3, 3, 3, 3, 1, 3, 2, 3, 3, 3, 3, 3, 2,
            2, 3, 1, 3, 0, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 1, 3, 3, 3, 3, 2, 2, 2, 3, 1, 1, 3, 2, 2,
            0, 3, 2, 1, 2, 1, 0, 3, 3, 3, 2, 2, 3, 2, 1, 2, 0, 0, 3, 3, 2, 3, 3, 2, 3, 3, 3, 3, 3,
            2, 2, 2, 3, 3, 3, 3, 3, 1, 1, 3, 2, 2, 3, 1, 1, 0, 1, 3, 2, 3, 3, 2, 3, 3, 2, 3, 3, 2,
            2, 2, 2, 3, 2, 2, 2, 2, 2, 1, 2, 3, 3, 2, 2, 2, 2, 3, 3, 2, 0, 2, 1, 3, 3, 3, 3, 0, 3,
            3, 3, 3, 2, 2, 3, 1, 3, 3, 3, 2, 3, 3, 3, 2, 3, 3, 3, 3, 2, 3, 2, 1, 3, 3, 3, 3, 2, 2,
            0, 1, 2, 3, 2, 0, 3, 3, 3, 3, 3, 3, 1, 3, 3, 2, 3, 2, 2, 3, 3, 3, 3, 3, 2, 2, 3, 3, 2,
            2, 2, 1, 3, 3, 3, 3, 1, 2, 3, 2, 3, 3, 2, 3, 2, 3, 3, 3, 2, 3, 1, 2, 3, 2, 1, 1, 3, 3,
            2, 3, 3, 2, 3, 3, 0, 0, 1, 3, 3, 2, 3, 3, 3, 3, 1, 3, 3, 0, 3, 2, 3, 3, 1, 3, 3, 3, 3,
            3, 3, 3, 0, 3, 3, 2,
        ];
        for e in expected_samples {
            assert_eq!(sampler.next(&mut xoshiro), e);
        }
    }

    #[test]
    fn test_choose_degree() {
        let message = crate::xoshiro::test_utils::make_message("Wolf", 1024);
        let fragment_length = crate::fountain::fragment_length(message.len(), 100);
        let fragments = crate::fountain::partition(message, fragment_length);
        let expected_degrees = vec![
            11, 3, 6, 5, 2, 1, 2, 11, 1, 3, 9, 10, 10, 4, 2, 1, 1, 2, 1, 1, 5, 2, 4, 10, 3, 2, 1,
            1, 3, 11, 2, 6, 2, 9, 9, 2, 6, 7, 2, 5, 2, 4, 3, 1, 6, 11, 2, 11, 3, 1, 6, 3, 1, 4, 5,
            3, 6, 1, 1, 3, 1, 2, 2, 1, 4, 5, 1, 1, 9, 1, 1, 6, 4, 1, 5, 1, 2, 2, 3, 1, 1, 5, 2, 6,
            1, 7, 11, 1, 8, 1, 5, 1, 1, 2, 2, 6, 4, 10, 1, 2, 5, 5, 5, 1, 1, 4, 1, 1, 1, 3, 5, 5,
            5, 1, 4, 3, 3, 5, 1, 11, 3, 2, 8, 1, 2, 1, 1, 4, 5, 2, 1, 1, 1, 5, 6, 11, 10, 7, 4, 7,
            1, 5, 3, 1, 1, 9, 1, 2, 5, 5, 2, 2, 3, 10, 1, 3, 2, 3, 3, 1, 1, 2, 1, 3, 2, 2, 1, 3, 8,
            4, 1, 11, 6, 3, 1, 1, 1, 1, 1, 3, 1, 2, 1, 10, 1, 1, 8, 2, 7, 1, 2, 1, 9, 2, 10, 2, 1,
            3, 4, 10,
        ];
        for nonce in 1..=200 {
            let mut xoshiro = crate::xoshiro::Xoshiro256::from(format!("Wolf-{nonce}").as_str());
            assert_eq!(
                xoshiro.choose_degree(fragments.len()),
                expected_degrees[nonce - 1]
            );
        }
    }

    #[test]
    #[should_panic(expected = "negative probability encountered")]
    fn test_negative_weights() {
        Weighted::new(vec![2.0, -1.0]);
    }

    #[test]
    #[should_panic(expected = "probabilities don't sum to a positive value")]
    fn test_zero_weights() {
        Weighted::new(vec![0.0]);
    }
}
