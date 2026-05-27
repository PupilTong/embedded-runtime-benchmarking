use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crate::cases::{self, Case};

const DEFAULT_WORKERS: usize = 4;
static WORKERS: AtomicUsize = AtomicUsize::new(DEFAULT_WORKERS);

pub fn set_workers(workers: usize) {
    WORKERS.store(workers.max(1), Ordering::Relaxed);
}

pub const CASES: &[Case] = &[
    Case {
        name: "Richards",
        default_iterations: 2_400,
        run: richards,
    },
    Case {
        name: "DeltaBlue",
        default_iterations: 700,
        run: delta_blue,
    },
    Case {
        name: "Crypto",
        default_iterations: 160,
        run: crypto,
    },
    Case {
        name: "RayTrace",
        default_iterations: 26,
        run: ray_trace,
    },
    Case {
        name: "EarleyBoyer",
        default_iterations: 36,
        run: earley_boyer,
    },
    Case {
        name: "RegExp",
        default_iterations: 180,
        run: regexp,
    },
    Case {
        name: "Splay",
        default_iterations: 520,
        run: splay,
    },
    Case {
        name: "NavierStokes",
        default_iterations: 34,
        run: navier_stokes,
    },
];

fn richards(iterations: u64) -> u64 {
    run_parallel(iterations, cases::richards)
}

fn delta_blue(iterations: u64) -> u64 {
    run_parallel(iterations, cases::delta_blue)
}

fn crypto(iterations: u64) -> u64 {
    run_parallel(iterations, cases::crypto)
}

fn ray_trace(iterations: u64) -> u64 {
    run_parallel(iterations, cases::ray_trace)
}

fn earley_boyer(iterations: u64) -> u64 {
    run_parallel(iterations, cases::earley_boyer)
}

fn regexp(iterations: u64) -> u64 {
    run_parallel(iterations, cases::regexp)
}

fn splay(iterations: u64) -> u64 {
    run_parallel(iterations, cases::splay)
}

fn navier_stokes(iterations: u64) -> u64 {
    run_parallel(iterations, cases::navier_stokes)
}

fn run_parallel(iterations: u64, run_case: fn(u64) -> u64) -> u64 {
    let worker_count = WORKERS
        .load(Ordering::Relaxed)
        .min(iterations.try_into().unwrap_or(usize::MAX))
        .max(1);

    if worker_count == 1 {
        return run_case(iterations);
    }

    let base_iterations = iterations / worker_count as u64;
    let extra_iterations = iterations % worker_count as u64;

    thread::scope(|scope| {
        let handles: Vec<_> = (0..worker_count)
            .map(|worker| {
                let worker_iterations =
                    base_iterations + u64::from(worker < extra_iterations as usize);
                scope.spawn(move || {
                    let checksum = run_case(worker_iterations);
                    cases::mix(checksum ^ worker as u64, worker_iterations)
                })
            })
            .collect();

        handles
            .into_iter()
            .enumerate()
            .fold(0_u64, |acc, (worker, handle)| {
                let checksum = handle.join().expect("benchmark worker thread panicked");
                cases::mix(acc ^ worker as u64, checksum)
            })
    })
}
