use std::{
    env::args,
    time::{Duration, Instant},
};

use processor::profile::read_timer_cpu;

fn main() {
    let millis_to_wait = args()
        .nth(1)
        .and_then(|t| t.parse::<u64>().ok())
        .unwrap_or(1000);

    let os_start = Instant::now();
    let cpu_start = read_timer_cpu();

    let mut os_elapsed = Duration::default();
    let mut os_end = Instant::now();
    let wait_time = Duration::from_millis(millis_to_wait);
    while wait_time > os_elapsed {
        os_end = Instant::now();
        os_elapsed = os_end.saturating_duration_since(os_start);
    }

    let cpu_end = read_timer_cpu();
    let cpu_elapsed = cpu_end - cpu_start;
    let cpu_frequency = cpu_elapsed as f64 / os_elapsed.as_secs_f64();

    println!("  OS Timer: {os_start:?} -> {os_end:?} = {os_elapsed:?} elapsed",);
    println!("OS Seconds: {}", os_elapsed.as_secs_f64());

    println!(" CPU Timer: {cpu_start} -> {cpu_end} = {cpu_elapsed} elapsed");
    println!("  CPU Freq: {cpu_frequency} (guessed)");
}
