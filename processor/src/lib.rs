pub mod profile {

    const D_WIDTH: usize = 12;
    const PREC: usize = 2;
    const P_WIDTH: usize = 3 + PREC;
    const L_WIDTH: usize = 24;

    use std::{
        arch::x86_64::_rdtsc,
        time::{Duration, Instant},
    };

    #[cfg(feature = "profile")]
    use self::internals::{Anchor, MAX_ANCHORS};

    pub static mut PROFILER: Profiler = Profiler::new();

    #[cfg(feature = "profile")]
    pub struct Profiler {
        pub tsc_start: u64,
        pub tsc_end: u64,
        pub anchors: [Anchor; MAX_ANCHORS],
        current_anchor: usize,
    }

    #[cfg(not(feature = "profile"))]
    pub struct Profiler {
        pub tsc_start: u64,
        pub tsc_end: u64,
    }

    impl Profiler {
        #[cfg(feature = "profile")]
        const fn new() -> Profiler {
            Self {
                tsc_start: 0,
                tsc_end: 0,
                anchors: [Anchor::blank(); MAX_ANCHORS],
                current_anchor: 0,
            }
        }

        #[cfg(not(feature = "profile"))]
        const fn new() -> Profiler {
            Self {
                tsc_start: 0,
                tsc_end: 0,
            }
        }
    }

    #[cfg(feature = "profile")]
    pub mod internals {

        use super::{read_timer_cpu, PROFILER};

        pub const MAX_ANCHORS: usize = 2048;

        fn get_anchor(index: usize) -> &'static mut Anchor {
            assert!(index < 2048);
            unsafe { PROFILER.anchors.get_unchecked_mut(index) }
        }

        #[derive(Debug, Default, Clone, Copy)]
        pub struct Anchor {
            pub hit_count: u64,
            pub label: &'static str,
            pub tsc_elapsed_inclusive: u64,
            pub tsc_elapsed_exclusive: u64,
        }

        impl Anchor {
            const BLANK_LABEL: &'static str = "";
            pub const fn blank() -> Self {
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

                parent.tsc_elapsed_exclusive =
                    parent.tsc_elapsed_exclusive.wrapping_sub(tsc_elapsed);
                anchor.tsc_elapsed_exclusive =
                    anchor.tsc_elapsed_exclusive.wrapping_add(tsc_elapsed);
                anchor.tsc_elapsed_inclusive = tsc_elapsed + self.elapsed_before;
                anchor.hit_count += 1;
            }
        }
    }

    pub fn get_os_timer_frequency() -> Duration {
        Duration::from_secs(1)
    }

    pub fn read_timer_cpu() -> u64 {
        unsafe { _rdtsc() }
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
    #[cfg(not(feature = "profile"))]
    pub fn begin_profile() {
        unsafe { PROFILER.tsc_start = read_timer_cpu() };
    }

    #[cfg(not(feature = "profile"))]
    pub fn measure_block(_id: usize, _label: &'static str) -> () {}

    #[cfg(not(feature = "profile"))]
    pub fn stop_and_print_timings() {
        unsafe { PROFILER.tsc_end = read_timer_cpu() };

        let tsc_end = unsafe { PROFILER.tsc_end };
        let tsc_start = unsafe { PROFILER.tsc_start };
        let elapsed_full = tsc_end - tsc_start;

        let freq = estimate_cpu_frequency(100);

        let elapsed_millis = (1000 * elapsed_full) as f64 / freq as f64;
        println!(
            "\nTotal time: {elapsed_millis:>P_WIDTH$.PREC$} ms {elapsed_full} (Cpu freq {freq})\n"
        );
    }

    #[cfg(feature = "profile")]
    pub fn begin_profile() {
        unsafe { PROFILER.tsc_start = read_timer_cpu() };
    }

    #[cfg(feature = "profile")]
    pub fn measure_block(id: usize, label: &'static str) -> internals::DropTimer {
        internals::DropTimer::start(id, label)
    }

    #[cfg(feature = "profile")]
    pub fn stop_and_print_timings() {
        unsafe { PROFILER.tsc_end = read_timer_cpu() };

        let tsc_end = unsafe { PROFILER.tsc_end };
        let tsc_start = unsafe { PROFILER.tsc_start };
        let elapsed_full = tsc_end - tsc_start;

        let freq = estimate_cpu_frequency(100);

        let elapsed_millis = (1000 * elapsed_full) as f64 / freq as f64;
        println!(
            "\nTotal time: {elapsed_millis:>P_WIDTH$.PREC$} ms {elapsed_full} (Cpu freq {freq})\n"
        );

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
}
