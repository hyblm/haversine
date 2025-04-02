pub mod profile {
    use std::{
        arch::x86_64::_rdtsc,
        time::{Duration, Instant},
    };

    pub fn estimate_cpu_frequency(millis_to_wait: u64) -> u64 {
        let os_start = Instant::now();
        let cpu_start = read_timer_cpu();

        let mut os_end;
        let mut os_elapsed = Duration::default();
        let wait_time = Duration::from_millis(millis_to_wait);
        while wait_time > os_elapsed {
            os_end = Instant::now();
            os_elapsed = os_end.saturating_duration_since(os_start);
        }

        let cpu_end = read_timer_cpu();
        let cpu_elapsed = cpu_end - cpu_start;
        let cpu_frequency = (cpu_elapsed as f64 / os_elapsed.as_secs_f64()) as u64;

        return cpu_frequency;
    }

    pub fn get_os_timer_frequency() -> Duration {
        Duration::from_secs(1)
    }

    pub fn read_timer_cpu() -> u64 {
        unsafe { _rdtsc() }
    }
}
