use std::time::{Duration, Instant};

use processor::profile::{get_os_timer_frequency, read_timer_cpu};

fn main() {
    let frequency = get_os_timer_frequency();

    let os_start = Instant::now();
    let cpu_start = read_timer_cpu();

    let mut os_elapsed = Duration::default();
    let mut os_end = Instant::now();
    while frequency > os_elapsed {
        os_end = Instant::now();
        os_elapsed = os_end.saturating_duration_since(os_start);
    }

    let cpu_end = read_timer_cpu();
    let cpu_elapsed = cpu_end - cpu_start;

    println!("  OS Timer: {os_start:?} -> {os_end:?} = {os_elapsed:?} elapsed",);
    println!("OS Seconds: {}", os_elapsed.as_secs_f64());

    println!(" CPU Timer: {cpu_start} -> {cpu_end} = {cpu_elapsed} elapsed");
}
