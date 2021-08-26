#[derive(Debug)]
pub struct Weighted {
    aliases: Vec<u32>,
    probs: Vec<f64>,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
impl Weighted {
    pub fn new(mut weights: Vec<f64>) -> anyhow::Result<Self> {
        if weights.iter().any(|p| *p < 0.0) {
            return Err(anyhow::anyhow!("negative probability encountered"));
        }
        let summed = weights.iter().sum::<f64>();
        if summed <= 0.0 {
            return Err(anyhow::anyhow!(
                "probabilities don't sum to a positive value"
            ));
        }
        let count = weights.len();
        for w in &mut weights {
            *w *= count as f64 / summed;
        }
        let mut s: Vec<usize> = Vec::with_capacity(count);
        let mut l: Vec<usize> = Vec::with_capacity(count);
        for j in 1..=count {
            let i = count - j;
            if *weights
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?
                < 1.0
            {
                s.push(i);
            } else {
                l.push(i);
            }
        }

        let mut probs: Vec<f64> = vec![0.0; count];
        let mut aliases: Vec<u32> = vec![0; count];

        while !s.is_empty() && !l.is_empty() {
            let a = s.remove(s.len() - 1);
            let g = l.remove(l.len() - 1);
            *probs
                .get_mut(a)
                .ok_or_else(|| anyhow::anyhow!("expected item"))? = *weights
                .get(a)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?;
            *aliases
                .get_mut(a)
                .ok_or_else(|| anyhow::anyhow!("expected item"))? = g as u32;
            *weights
                .get_mut(g)
                .ok_or_else(|| anyhow::anyhow!("expected item"))? += *weights
                .get(a)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?
                - 1.0;
            if *weights
                .get(g)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?
                < 1.0
            {
                s.push(g);
            } else {
                l.push(g);
            }
        }

        while !l.is_empty() {
            let g = l.remove(l.len() - 1);
            *probs
                .get_mut(g)
                .ok_or_else(|| anyhow::anyhow!("expected item"))? = 1.0;
        }

        while !s.is_empty() {
            let a = s.remove(s.len() - 1);
            *probs
                .get_mut(a)
                .ok_or_else(|| anyhow::anyhow!("expected item"))? = 1.0;
        }

        Ok(Self { aliases, probs })
    }

    #[allow(clippy::should_implement_trait)]
    #[allow(clippy::cast_sign_loss)]
    pub fn next(&mut self, xoshiro: &mut crate::xoshiro::Xoshiro256) -> anyhow::Result<u32> {
        let r1 = xoshiro.next_double();
        let r2 = xoshiro.next_double();
        let n = self.probs.len();
        let i = (n as f64 * r1) as usize;
        if r2
            < *self
                .probs
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?
        {
            Ok(i as u32)
        } else {
            Ok(*self
                .aliases
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("expected item"))?)
        }
    }
}
