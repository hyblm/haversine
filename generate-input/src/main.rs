use std::{ops::RangeInclusive, str::FromStr};

use cli::Generation;
use rand::{rngs::SmallRng, SeedableRng};

mod cli;

fn main() {
    let Some(settings) = cli::run() else { return };
    let (expected_sum, pairs): (f64, _) = generate_pairs(&settings);
    println!("Expected sum: {expected_sum}");

    let json = convert_to_json(&pairs);
    println!("{json}");
}

const CLUSTER_COUNT: usize = 100;
const CLUSTER_SIZE: f64 = 20.;

fn generate_pairs(settings: &cli::Settings) -> (f64, Vec<(Coordinate, Coordinate)>) {
    let mut rng = SmallRng::seed_from_u64(settings.generation_seed);

    let mut pairs = Vec::with_capacity(settings.requested_count);
    match settings.generation_type {
        Generation::Uniform => {
            for _ in 0..(settings.requested_count) {
                pairs.push((Coordinate::random(&mut rng), Coordinate::random(&mut rng)));
            }
        }

        Generation::Cluster => {
            let mut cluster_origin = Coordinate { x: 0., y: 0. };
            for i in 0..(settings.requested_count) {
                if (i % CLUSTER_COUNT) == 0 {
                    cluster_origin = Coordinate::random(&mut rng);
                }
                pairs.push((
                    Coordinate::random_in_cluster(&mut rng, cluster_origin),
                    Coordinate::random_in_cluster(&mut rng, cluster_origin),
                ));
            }
        }
    }

    (0., pairs)
}

/// produces JSON in this form:
/// ```json
/// {"pairs": [
///     { "x0": 3.45, "y0": 1.68, "x1": 2.83, "y1": 2.16 },
///     ...
/// ]}
/// ```
fn convert_to_json(pairs: &[(Coordinate, Coordinate)]) -> String {
    let mut json = String::from_str("{\"pairs\": [").unwrap();
    for pair in pairs {
        json.push_str(&format!(
            "\n  {{ \"x0\": {}, \"y0\": {}, \"x1\": {}, \"y1\": {} }},",
            pair.0.x, pair.0.y, pair.1.x, pair.1.y
        ));
    }
    json.pop();
    json.push_str("\n]}");
    json
}

#[derive(Debug, Clone, Copy)]
struct Coordinate {
    x: f64,
    y: f64,
}

impl Coordinate {
    pub const X_RANGE: RangeInclusive<f64> = -180.0..=180.0;
    pub const Y_RANGE: RangeInclusive<f64> = -90.0..=90.0;

    fn random(mut rng: impl rand::Rng) -> Self {
        Self {
            x: rng.random_range(Self::X_RANGE),
            y: rng.random_range(Self::Y_RANGE),
        }
    }

    fn random_in_cluster(mut rng: impl rand::Rng, origin: Coordinate) -> Self {
        Self {
            x: (rng.random_range(Self::X_RANGE) % CLUSTER_SIZE) + origin.x,
            y: (rng.random_range(Self::Y_RANGE) % CLUSTER_SIZE) + origin.y,
        }
    }
}
