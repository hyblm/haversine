use std::io::Read;

use haversine::reference_haversine;
use processor::profile::{self, begin_profile};

fn main() {
    begin_profile();

    let Some((mut file_input, _file_answers)) = cli::run() else {
        return;
    };

    let json = {
        let _x = profile::measure_block(1, "Read Entire File");

        let mut json = String::new();
        if let Err(error) = file_input.read_to_string(&mut json) {
            eprintln!("failed to read file with error: {error}");
            return;
        }
        json
    };

    let Some(pairs) = json_parse(&json) else {
        return;
    };

    let haversine_average = {
        let _x = profile::measure_block(2, "Sum Haversine Distances");
        let haversine_sum: f64 = pairs
            .iter()
            .map(|&(x0, y0, x1, y1)| reference_haversine(x0, y0, x1, y1, haversine::EARTH_RADIUS))
            .sum();
        haversine_sum / pairs.len() as f64
    };

    println!("Input size: {}", json.len());
    println!("Pair count: {}", pairs.len());
    println!("Average: {haversine_average}",);

    profile::stop_and_print_timings();
}

fn json_parse(json: &str) -> Option<Vec<(f64, f64, f64, f64)>> {
    let _x = profile::measure_block(3, "Parse Haversine Pairs");

    let (start, json) = json.split_once('[')?;
    let start = start.split_whitespace();
    let mut required_tokens = "{\"pairs\":".chars();
    for item in start {
        for char in item.chars() {
            let required_token = required_tokens.next()?;
            if char != required_token {
                eprintln!("Failed to parse on {char} != {required_token}");
                return None;
            }
        }
    }

    let mut pairs = Vec::new();
    for item in json.split('}') {
        let _x = profile::measure_block(4, "Parse Coordinate Pair");
        // end of the array
        if item.find(']').is_some() {
            break;
        };

        let offset = item.find('{')?;
        let rest = &item[offset + 1..];

        let offset = rest.find("\"x0\"")?;
        let rest = &rest[offset + 1..];
        let offset = rest.find(':')?;
        let rest = &rest[offset + 1..];
        let offset_comma = rest.find(',')?;
        let x0: f64 = rest[..offset_comma].trim().parse().ok()?;
        let rest = &rest[offset_comma + 1..];

        let offset = rest.find("\"y0\"")?;
        let rest = &rest[offset + 1..];
        let offset = rest.find(':')?;
        let rest = &rest[offset + 1..];
        let offset_comma = rest.find(',')?;
        let y0: f64 = rest[..offset_comma].trim().parse().ok()?;
        let rest = &rest[offset_comma + 1..];

        let offset = rest.find("\"x1\"")?;
        let rest = &rest[offset + 1..];
        let offset = rest.find(':')?;
        let rest = &rest[offset + 1..];
        let offset_comma = rest.find(',')?;
        let x1: f64 = rest[..offset_comma].trim().parse().ok()?;
        let rest = &rest[offset_comma + 1..];

        let offset = rest.find("\"y1\"")?;
        let rest = &rest[offset + 1..];
        let offset = rest.find(':')?;
        let rest = &rest[offset + 1..];
        let y1: f64 = rest.trim().parse().ok()?;

        pairs.push((x0, y0, x1, y1));
    }

    Some(pairs)
}

mod cli {
    use std::{env::args, fs::File};

    pub fn run() -> Option<(File, Option<File>)> {
        let mut args = args();

        let name = args.next()?;
        let result = fun_name(args);
        if result.is_none() {
            print_usage(&name);
        }
        result
    }

    fn fun_name(mut args: std::env::Args) -> Option<(File, Option<File>)> {
        let file_path_input = args.next()?;
        let file_path_answers = args.next();

        let file_input = File::open(file_path_input).ok()?;
        let file_answers_maybe = file_path_answers.and_then(|path| File::open(path).ok());

        Some((file_input, file_answers_maybe))
    }

    fn print_usage(name: &str) {
        eprintln!(
            "
Usage: {name} [haversine_input.json]
       {name} [haversine_input.json] [answers.f64]
            "
        );
    }
}
