pub use ts_common::{Edge, Vertex, Weight};

mod ts_common {
    use anyhow::{Context, Result, bail};
    use rand::{distr::Uniform, prelude::*};
    use rand_distr::Normal;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Vertex(pub u16);

    impl Vertex {
        pub fn random(rng: &mut impl Rng, num_vertices: u16) -> Result<Self> {
            Ok(Self(rng.sample(
                Uniform::new(0u16, num_vertices).context("num_vertices must be non-zero")?,
            )))
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Weight(pub u16);

    impl Weight {
        pub fn random(rng: &mut impl Rng, mean: u16, sd: u16) -> Result<Self> {
            Ok(Self(
                rng.sample(Normal::new(mean as f64, sd as f64).context("SD must be finite")?)
                    .clamp(1.0, u16::MAX as f64) as u16,
            ))
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Edge(pub Vertex, pub Vertex, pub Weight);

    impl Edge {
        pub fn random(
            rng: &mut impl Rng,
            num_vertices: u16,
            weight_mean: u16,
            weight_sd: u16,
        ) -> Result<Self> {
            if num_vertices <= 1 {
                bail!("At least two vertices are needed to make an edge");
            }
            loop {
                let v = Vertex::random(rng, num_vertices)?;
                let u = Vertex::random(rng, num_vertices)?;
                if v == u {
                    continue;
                }
                return Ok(Self(v, u, Weight::random(rng, weight_mean, weight_sd)?));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_edge_ok() {
        let mut rng = rand::rng();
        let edge = Edge::random(&mut rng, 10, 10, 5).unwrap();
        assert_ne!(edge.0.0, edge.1.0);

        let edge = Edge::random(&mut rng, 2, 10, 5).unwrap();
        assert_ne!(edge.0.0, edge.1.0);
    }

    #[test]
    fn random_edge_err() {
        let mut rng = rand::rng();
        assert!(Edge::random(&mut rng, 1, 10, 5).is_err());
    }

    #[test]
    fn random_vertex_err() {
        let mut rng = rand::rng();
        assert!(Vertex::random(&mut rng, 0).is_err());
    }

    #[test]
    fn random_vertex_ok() {
        let mut rng = rand::rng();
        assert!(Vertex::random(&mut rng, 1).is_ok());
    }

    #[test]
    fn random_weight_ok() {
        let mut rng = rand::rng();
        let Weight(w) = Weight::random(&mut rng, 10, 10).unwrap();
        assert!(w >= 1);

        let Weight(w) = Weight::random(&mut rng, 10, 0).unwrap();
        assert_eq!(w, 10);
    }
}
