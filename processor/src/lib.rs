pub mod profile {
    use std::{
        arch::x86_64::_rdtsc,
        time::{Duration, Instant},
    };

    const MAX_ANCHORS: usize = 2048;
    static mut PROFILER: Profiler = Profiler::new();

    struct Profiler {
        tsc_start: u64,
        tsc_end: u64,
        anchors: [Anchor; MAX_ANCHORS],
        current_anchor: usize,
    }

    impl Profiler {
        const fn new() -> Profiler {
            Self {
                tsc_start: 0,
                tsc_end: 0,
                anchors: [Anchor::blank(); MAX_ANCHORS],
                current_anchor: 0,
            }
        }
    }

    pub fn begin_profile() {
        unsafe { PROFILER.tsc_start = read_timer_cpu() };
    }

    fn get_anchor(index: usize) -> &'static mut Anchor {
        assert!(index < 2048);
        unsafe { PROFILER.anchors.get_unchecked_mut(index) }
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct Anchor {
        hit_count: u64,
        label: &'static str,
        tsc_elapsed_inclusive: u64,
        tsc_elapsed_exclusive: u64,
    }

    impl Anchor {
        const BLANK_LABEL: &'static str = "";
        const fn blank() -> Self {
            Self {
                hit_count: 0,
                label: Self::BLANK_LABEL,
                tsc_elapsed_inclusive: 0,
                tsc_elapsed_exclusive: 0,
            }
        }
    }

    pub struct DropTimer {
        label: &'static str,
        cpu_start: u64,
        anchor_index: usize,
        parent_index: usize,
        elapsed_before: u64,
    }
    impl DropTimer {
        pub fn start(id: usize, label: &'static str) -> DropTimer {
            assert_ne!(id, 0);

            let anchor_index = id;
            let parent_index = unsafe { PROFILER.current_anchor };
            unsafe { PROFILER.current_anchor = anchor_index };

            let anchor = get_anchor(anchor_index);
            let elapsed_before = anchor.tsc_elapsed_inclusive;

            let cpu_start = read_timer_cpu();
            DropTimer {
                label,
                cpu_start,
                anchor_index,
                parent_index,
                elapsed_before,
            }
        }
    }

    impl Drop for DropTimer {
        fn drop(&mut self) {
            let tsc_elapsed = read_timer_cpu() - self.cpu_start;
            unsafe { PROFILER.current_anchor = self.parent_index };

            let parent = get_anchor(self.parent_index);
            let anchor = get_anchor(self.anchor_index);

            // NOTE (matyas): This is a hacky way of forcing the user to use unique id's for each
            //                callsite. Ideally we could generate a unique id per callsite for them,
            //                but that seems to be quite cumbersome to do in Rust so we're going
            //                with this as *temporary* solution for now.
            if anchor.label != Anchor::BLANK_LABEL {
                assert_eq!(anchor.label, self.label);
            } else {
                anchor.label = self.label;
            }

            // let tsc_elapsed_exclusive = tsc_elapsed - anchor.tsc_elapsed_children;

            parent.tsc_elapsed_exclusive = parent.tsc_elapsed_exclusive.wrapping_sub(tsc_elapsed);
            anchor.tsc_elapsed_exclusive = anchor.tsc_elapsed_exclusive.wrapping_add(tsc_elapsed);
            anchor.tsc_elapsed_inclusive = tsc_elapsed + self.elapsed_before;
            anchor.hit_count += 1;
        }
    }

    const D_WIDTH: usize = 12;
    const PREC: usize = 2;
    const P_WIDTH: usize = 3 + PREC;
    const L_WIDTH: usize = 24;

    pub fn stop_and_print_timings() {
        unsafe { PROFILER.tsc_end = read_timer_cpu() };
        let elapsed_full = unsafe { PROFILER.tsc_end - PROFILER.tsc_start };
        let freq = estimate_cpu_frequency(100);

        println!("\nTotal time: {elapsed_full} (Cpu freq {freq})\n");

        let anchors = unsafe { PROFILER.anchors.iter() };
        for anchor in anchors.skip(1) {
            if anchor.hit_count == 0 {
                break;
            }

            let percent_exclusive =
                (100 * anchor.tsc_elapsed_exclusive) as f64 / elapsed_full as f64;
            print!(
                "{:>5}  {:>L_WIDTH$}:  {percent_exclusive:>P_WIDTH$.PREC$}% {:>D_WIDTH$}",
                anchor.hit_count, anchor.label, anchor.tsc_elapsed_exclusive
            );

            if anchor.tsc_elapsed_exclusive != anchor.tsc_elapsed_inclusive {
                let percent_full =
                    (100 * anchor.tsc_elapsed_inclusive) as f64 / elapsed_full as f64;
                println!(
                    " > {percent_full:>P_WIDTH$.PREC$}% {:>D_WIDTH$}",
                    anchor.tsc_elapsed_inclusive
                );
            } else {
                println!();
            }
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
