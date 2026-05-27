mod cases;
#[cfg(wasip1_threads)]
mod threaded_cases;

use std::env;
use std::time::Instant;

struct Options {
    samples: usize,
    scale: u64,
    case_filter: Option<String>,
    threaded: bool,
    workers: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            samples: 5,
            scale: 1,
            case_filter: None,
            threaded: false,
            workers: 4,
        }
    }
}

fn parse_args() -> Options {
    let mut options = Options::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--samples" => {
                if let Some(value) = args.next() {
                    options.samples = value.parse().unwrap_or(options.samples).max(1);
                }
            }
            "--scale" => {
                if let Some(value) = args.next() {
                    options.scale = value.parse().unwrap_or(options.scale).max(1);
                }
            }
            "--case" => {
                options.case_filter = args.next().map(|case| case.to_ascii_lowercase());
            }
            "--threads" => {
                options.threaded = true;
            }
            "--workers" => {
                if let Some(value) = args.next() {
                    options.workers = value.parse().unwrap_or(options.workers).max(1);
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
    }

    options
}

fn print_help() {
    println!(
        "Usage: embedded-runtime-benchmarking [--samples N] [--scale N] [--case NAME] [--threads] [--workers N]"
    );
}

fn main() {
    let options = parse_args();
    let (mode, worker_count, selected_cases) = if options.threaded {
        threaded_cases_for(options.workers)
    } else {
        ("rust", 1, cases::CASES)
    };

    println!(
        "{{\"event\":\"suite\",\"language\":\"rust\",\"mode\":\"{}\",\"target_arch\":\"{}\",\"target_os\":\"{}\",\"samples\":{},\"scale\":{},\"workers\":{}}}",
        mode,
        env::consts::ARCH,
        env::consts::OS,
        options.samples,
        options.scale,
        worker_count
    );

    for case in selected_cases {
        if let Some(filter) = &options.case_filter {
            if case.name.to_ascii_lowercase() != *filter {
                continue;
            }
        }

        let iterations = case.default_iterations.saturating_mul(options.scale);
        for sample in 0..options.samples {
            let start = Instant::now();
            let checksum = std::hint::black_box((case.run)(iterations));
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

            println!(
                "{{\"event\":\"sample\",\"case\":\"{}\",\"sample\":{},\"iterations\":{},\"elapsed_ms\":{:.6},\"checksum\":{}}}",
                case.name,
                sample + 1,
                iterations,
                elapsed_ms,
                checksum
            );
        }
    }
}

#[cfg(wasip1_threads)]
fn threaded_cases_for(workers: usize) -> (&'static str, usize, &'static [cases::Case]) {
    threaded_cases::set_workers(workers);
    ("rust-threads", workers, threaded_cases::CASES)
}

#[cfg(not(wasip1_threads))]
fn threaded_cases_for(_: usize) -> (&'static str, usize, &'static [cases::Case]) {
    eprintln!("--threads requires the wasm32-wasip1-threads target with atomics enabled");
    std::process::exit(2);
}
