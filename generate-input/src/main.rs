use std::{fs::File, io::Write, ops::RangeInclusive, process::exit, str::FromStr};

use cli::Generation;
use haversine::reference_haversine;
use rand::{rngs::SmallRng, SeedableRng};

mod cli;

fn main() {
    let Some(settings) = cli::run() else { return };
    let pairs = generate_pairs(&settings);
    let haversine_sum: f64 = pairs
        .iter()
        .map(
            |&(Coordinate { x: x0, y: y0 }, Coordinate { x: x1, y: y1 })| {
                reference_haversine(x0, y0, x1, y1, haversine::EARTH_RADIUS)
            },
        )
        .sum();
    let haversine_average = haversine_sum / pairs.len() as f64;
    println!("Expected sum: {haversine_average}");

    let json = convert_to_json(&pairs);
    let mut file = match File::create("out.json") {
        Ok(file) => file,
        Err(error) => {
            eprintln!("couldn't create output file: {error}");
            exit(1);
        }
    };

    if let Err(error) = file.write_all(json.as_bytes()) {
        eprintln!("couldn't write to output file: {error}");
        exit(1);
    };
}

const CLUSTER_SIZE: f64 = 20.;

fn generate_pairs(settings: &cli::Settings) -> Vec<(Coordinate, Coordinate)> {
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
            let cluster_count = 1 + (settings.requested_count / 64);
            for i in 0..(settings.requested_count) {
                if (i % cluster_count) == 0 {
                    cluster_origin = Coordinate::random(&mut rng);
                }
                pairs.push((
                    Coordinate::random_in_cluster(&mut rng, cluster_origin),
                    Coordinate::random_in_cluster(&mut rng, cluster_origin),
                ));
            }
        }
    }

    pairs
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
