use std::{
    env::{args, Args},
    fmt::Display,
};

pub fn run() -> Option<Settings> {
    let mut args = args();
    let name = args.next().unwrap_or(format!("generate-input"));
    let settings = parse_settings(args);
    if settings.is_none() {
        print_usage(&name);
    };
    settings
}

const MAX_PAIRS: usize = 1 << 34;

pub struct Settings {
    pub generation_type: Generation,
    pub generation_seed: u64,
    pub requested_count: usize,
}

pub enum Generation {
    Uniform,
    Cluster,
}

impl Display for Generation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Generation::Uniform => write!(f, "uniform"),
            Generation::Cluster => write!(f, "cluster"),
        }
    }
}

fn parse_settings(mut args: Args) -> Option<Settings> {
    let generation_type = match args.next()?.as_str() {
        "uniform" => Generation::Uniform,
        "cluster" => Generation::Cluster,
        _ => return None,
    };

    let generation_seed = args.next()?.parse().ok()?;
    let requested_count: usize = args.next()?.parse().ok()?;
    let requested_count = requested_count.min(MAX_PAIRS);

    println!(
        "
Generation Type: {generation_type}
Generation Seed: {generation_seed}
Requested count: {requested_count}
"
    );

    Some(Settings {
        generation_type,
        generation_seed,
        requested_count,
    })
}

fn print_usage(name: &str) {
    eprintln!(
        "Usage: {name} [uniform/cluster] [random seed] [number of coordinate pairs to generate]"
    );
}
