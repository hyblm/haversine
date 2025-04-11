pub mod profile {
    use std::{
        arch::x86_64::_rdtsc,
        time::{Duration, Instant},
    };

    const MAX_ANCHORS: usize = 10;

    static mut START: u64 = 0;
    static mut END: u64 = 0;
    static mut COUNTER: usize = 0;
    static mut TIMINGS: [ProfileAnchor; MAX_ANCHORS] = [ProfileAnchor::blank(); MAX_ANCHORS];

    fn counter() -> usize {
        unsafe { COUNTER += 1 };
        unsafe { COUNTER }
    }

    fn get_anchor(index: usize) -> &'static mut ProfileAnchor {
        assert!(index < 2048);
        unsafe { TIMINGS.get_unchecked_mut(index) }
    }

    pub fn begin_profile() {
        unsafe { START = read_timer_cpu() };
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct ProfileAnchor {
        hit_count: u64,
        tsc_elapsed: u64,
        label: &'static str,
    }

    impl ProfileAnchor {
        const BLANK_LABEL: &'static str = "Unused Anchor";
        const fn blank() -> Self {
            Self {
                hit_count: 0,
                tsc_elapsed: 0,
                label: Self::BLANK_LABEL,
            }
        }
    }

    pub struct DropTimer {
        cpu_start: u64,
        anchor_index: usize,
    }
    impl DropTimer {
        pub fn start(label: &'static str) -> DropTimer {
            let cpu_start = read_timer_cpu();

            let anchor_index = counter();
            let anchor = get_anchor(anchor_index);
            anchor.label = label;

            DropTimer {
                cpu_start,
                anchor_index,
            }
        }
    }

    impl Drop for DropTimer {
        fn drop(&mut self) {
            let anchor = get_anchor(self.anchor_index);
            anchor.tsc_elapsed = read_timer_cpu() - self.cpu_start;
            anchor.hit_count += 1;
        }
    }

    pub fn stop_and_print_timings() {
        unsafe { END = read_timer_cpu() };
        let elapsed_full = unsafe { END - START };
        let freq = estimate_cpu_frequency(100);

        println!("\nTotal time: {elapsed_full} (Cpu freq {freq})\n");

        let timings = unsafe { TIMINGS.iter() };
        for anchor in timings.skip(1) {
            if anchor.hit_count == 0 {
                break;
            }

            let d_width = 12;
            let prec = 2;
            let p_width = 3 + prec;
            let percentage = (100 * anchor.tsc_elapsed) as f64 / elapsed_full as f64;
            println!(
                "{}  {percentage:>p_width$.prec$}%  ( {:>d_width$} )   {}",
                anchor.hit_count, anchor.tsc_elapsed, anchor.label
            );
        }
    }

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
