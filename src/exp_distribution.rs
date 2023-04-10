use rand::distributions::Uniform;

#[derive(Copy, Clone)]
pub(crate) struct ExpDistribution {
    mean: f64,
}

impl ExpDistribution {
    pub fn new(mean: f64) -> Self {
        Self { mean }
    }
}

impl rand::distributions::Distribution<f64> for ExpDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let rand_value = rng.sample(Uniform::new(0f64, 1f64));
        -(1f64 - rand_value).ln() * self.mean
    }
}
