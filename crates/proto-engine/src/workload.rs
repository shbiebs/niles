//! Workload generation.
//!
//! NOTATION, stated explicitly because the literature is genuinely ambiguous here: the skew
//! parameter used throughout is the **Zipf rank exponent s**, where the probability of the
//! rank-r key is proportional to r^(-s). Higher s means *more* skew. This is NOT the Pareto
//! shape parameter, for which higher means a *thinner* tail and *less* skew. Reporting a
//! phase diagram against a bare "alpha" would invite a reader to read the axis backwards.

/// A Zipf(s) sampler over ranks 1..=n, using inverse-CDF lookup on a precomputed table.
pub struct Zipf {
    cdf: Vec<f64>,
    rng: Lcg,
}

impl Zipf {
    pub fn new(n: usize, s: f64, seed: u64) -> Self {
        let mut w = Vec::with_capacity(n);
        let mut total = 0.0;
        for r in 1..=n {
            let x = 1.0 / (r as f64).powf(s);
            total += x;
            w.push(total);
        }
        for v in w.iter_mut() {
            *v /= total;
        }
        Self {
            cdf: w,
            rng: Lcg::new(seed),
        }
    }

    /// Sample a 0-based key index.
    pub fn sample(&mut self) -> usize {
        let u = self.rng.next_f64();
        match self.cdf.binary_search_by(|p| p.partial_cmp(&u).unwrap()) {
            Ok(i) => i,
            Err(i) => i.min(self.cdf.len() - 1),
        }
    }

    pub fn rng(&mut self) -> &mut Lcg {
        &mut self.rng
    }
}

/// A small, explicit, reproducible PRNG. Using a named generator with a stated algorithm
/// (rather than a library default that may change) is what makes a seed a reproducibility
/// guarantee rather than a hope.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self(
            seed.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407),
        )
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / ((1u64 << 53) as f64)
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n.max(1)
    }
}
