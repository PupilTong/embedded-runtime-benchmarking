# Embedded Runtime Benchmarking

This repository benchmarks the V8 v7-style workload shape used by [ahaoboy/js-engine-benchmark](https://github.com/ahaoboy/js-engine-benchmark): Richards, DeltaBlue, Crypto, RayTrace, EarleyBoyer, RegExp, Splay, and NavierStokes. The JavaScript engines run the JS workload in `benchmarks/js/runner.js`; WAMR runs an idiomatic Rust 2024 rewrite compiled to WASI preview1 wasm.

## Current Machine

- Generated at: `2026-06-05T08:10:44+00:00`
- Host: `Darwin arm64`
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- WASI targets: `wasm32-wasip1` and `wasm32-wasip1-threads`
- Samples per case: `5`
- Scale: `1`
- Thread workers: `4`
- Valid sample rows: `360`

## Runtime Status

| Runtime | Binary | Version | Status |
| --- | --- | --- | --- |
| quickjs | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/quickjs/qjs | QuickJS version 2025-09-13 | ok |
| primjs | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/primjs/out/Default/qjs | PrimJS 2.11.1-rc.1 | ok |
| wamr-fast-interp | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/wasm-micro-runtime/product-mini/platforms/darwin/build/iwasm | iwasm 2.4.4 | ok |
| wasmi | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/wasmi-cli/bin/wasmi | wasmi 2.0.0-beta.2 | ok |
| wamr-fast-interp-threads | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/wasm-micro-runtime/product-mini/platforms/darwin/build-threads-mem/iwasm | iwasm 2.4.4 | ok |
| wasmtime-pulley | /opt/homebrew/bin/wasmtime | wasmtime 45.0.0 (377cd917a 2026-05-21) | ok |
| wasmtime-jit | /opt/homebrew/bin/wasmtime | wasmtime 45.0.0 (377cd917a 2026-05-21) | ok |
| wasmtime-pulley-tail | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/wasmtime-pulley-tail-nightly/bin/wasmtime | wasmtime 45.0.0 (Pulley tail-call loop; nightly build) | ok |
| wasmtime-pulley-threads | /opt/homebrew/bin/wasmtime | wasmtime 45.0.0 (377cd917a 2026-05-21) | failed: Error: the wasm_threads feature is not supported on this compiler configuration |
| wasmtime-jit-threads | /opt/homebrew/bin/wasmtime | wasmtime 45.0.0 (377cd917a 2026-05-21) | ok |
| wasmtime-pulley-tail-threads | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/wasmtime-pulley-tail-nightly/bin/wasmtime | wasmtime 45.0.0 (Pulley tail-call loop; nightly build) | failed: Error: the wasm_threads feature is not supported on this compiler configuration |

## Results

Median elapsed time in milliseconds. Lower is better.

| Case | quickjs | primjs | wamr-fast-interp | wasmi | wamr-fast-interp-threads | wasmtime-pulley | wasmtime-pulley-tail | wasmtime-jit | wasmtime-pulley-threads | wasmtime-pulley-tail-threads | wasmtime-jit-threads | Fastest | Fastest Interpreter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Richards | 22.000 | 33.000 | 4.780 | 4.464 | 1.525 | 5.365 | 6.386 | 0.458 | - | - | 0.200 | wasmtime-jit-threads | wamr-fast-interp-threads |
| DeltaBlue | 16.000 | 16.000 | 11.731 | 10.051 | 2.746 | 20.106 | 11.413 | 0.449 | - | - | 0.178 | wasmtime-jit-threads | wamr-fast-interp-threads |
| Crypto | 508.000 | 795.000 | 17.474 | 16.282 | 5.070 | 31.972 | 22.755 | 3.238 | - | - | 0.930 | wasmtime-jit-threads | wamr-fast-interp-threads |
| RayTrace | 33.000 | 40.000 | 17.532 | 9.687 | 3.683 | 20.759 | 10.105 | 0.536 | - | - | 0.205 | wasmtime-jit-threads | wamr-fast-interp-threads |
| EarleyBoyer | 14.000 | 23.000 | 7.882 | 9.006 | 18.723 | 12.007 | 13.094 | 0.607 | - | - | 1.607 | wasmtime-jit | wamr-fast-interp |
| RegExp | 82.000 | 111.000 | 71.205 | 57.644 | 18.789 | 79.172 | 78.285 | 3.392 | - | - | 0.924 | wasmtime-jit-threads | wamr-fast-interp-threads |
| Splay | 665.000 | 1291.000 | 5.971 | 7.896 | 3.183 | 9.285 | 10.747 | 0.939 | - | - | 0.523 | wasmtime-jit-threads | wamr-fast-interp-threads |
| NavierStokes | 36.000 | 42.000 | 14.858 | 14.049 | 3.279 | 29.802 | 17.748 | 1.017 | - | - | 0.316 | wasmtime-jit-threads | wamr-fast-interp-threads |

## V8 V7 Style Score

Higher is better. This table follows the score shape from `ahaoboy/js-engine-benchmark`'s V8 v7 harness: each case score is `100 * reference / median_us`, then `Score` is the geometric mean of the case scores. The reference constants are the `BenchmarkSuite(..., reference, ...)` values from the upstream V8 v7 case files. `Score/MB` follows upstream as `Score / Total size in MiB`, where `Total size = Exe size + Dll size` for the runtime binary. This repository uses median full-sample timings rather than the upstream one-second harness average, so the scores are intended for comparing rows in this report, not as official V8/Octane scores.

| Metric | wasmtime-jit-threads | wasmtime-jit | wamr-fast-interp-threads | wasmi | wamr-fast-interp | wasmtime-pulley-tail | wasmtime-pulley | quickjs | primjs | wasmtime-pulley-threads | wasmtime-pulley-tail-threads |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Version | wasmtime 45.0.0 (377cd917a 2026-05-21) | wasmtime 45.0.0 (377cd917a 2026-05-21) | iwasm 2.4.4 | wasmi 2.0.0-beta.2 | iwasm 2.4.4 | wasmtime 45.0.0 (Pulley tail-call loop; nightly build) | wasmtime 45.0.0 (377cd917a 2026-05-21) | QuickJS version 2025-09-13 | PrimJS 2.11.1-rc.1 | wasmtime 45.0.0 (377cd917a 2026-05-21) | wasmtime 45.0.0 (Pulley tail-call loop; nightly build) |
| Total size | 46M | 46M | 540.9K | 5.2M | 522K | 66.3M | 46M | 950.4K | 2.4M | 46M | 66.3M |
| Exe size | 46M | 46M | 540.9K | 5.2M | 522K | 66.3M | 46M | 950.4K | 2.4M | 46M | 66.3M |
| Dll size | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Richards | 17688 | 7708 | 2315 | 791 | 739 | 553 | 658 | 160 | 107 | - | - |
| DeltaBlue | 37241 | 14727 | 2408 | 658 | 564 | 579 | 329 | 413 | 413 | - | - |
| Crypto | 28610 | 8221 | 5250 | 1635 | 1523 | 1170 | 833 | 52.4 | 33.5 | - | - |
| RayTrace | 360092 | 137972 | 20092 | 7639 | 4221 | 7323 | 3565 | 2242 | 1850 | - | - |
| EarleyBoyer | 41479 | 109819 | 3560 | 7400 | 8456 | 5090 | 5551 | 4760 | 2898 | - | - |
| RegExp | 98623 | 26858 | 4849 | 1580 | 1279 | 1164 | 1151 | 1111 | 821 | - | - |
| Splay | 15575 | 8680 | 2560 | 1032 | 1365 | 758 | 878 | 12.3 | 6.31 | - | - |
| NavierStokes | 469374 | 145937 | 45258 | 10563 | 9988 | 8361 | 4980 | 4122 | 3533 | - | - |
| Score | 61436 | 28859 | 5738 | 2316 | 2111 | 1785 | 1434 | 462 | 331 | - | - |
| Score/MB | 1336 | 627 | 10864 | 444 | 4141 | 26 | 31 | 497 | 140 | - | - |

## Notes

- none

## How To Run

```sh
python3 scripts/bench.py --samples 5 --scale 1
```

Runtime binaries can be overridden with environment variables:

```sh
QUICKJS_BIN=/path/to/qjs PRIMJS_BIN=/path/to/primjs IWASM_BIN=/path/to/iwasm IWASM_THREADS_BIN=/path/to/iwasm WASMI_BIN=/path/to/wasmi WASMTIME_BIN=/path/to/wasmtime WASMTIME_PULLEY_TAIL_BIN=/path/to/wasmtime-tail WASM_OPT_BIN=/path/to/wasm-opt python3 scripts/bench.py
```

On macOS, Homebrew can provide the external optimizer/runtime tools:

```sh
brew install binaryen wasm-micro-runtime python@3.11 wasmtime
cargo install wasmi_cli --version 2.0.0-beta.2 --root tools/wasmi-cli --features simd --locked --force
```

QuickJS and PrimJS are expected to be source-built for this comparison, then passed through `QUICKJS_BIN` and `PRIMJS_BIN`. The current default source builds follow ahaoboy's engine package scripts:

- QuickJS: `tools/quickjs/qjs`, built from `https://github.com/bellard/quickjs.git` with `make`, then `strip qjs qjsc`, matching [`ahaoboy/quickjs-build`](https://github.com/ahaoboy/quickjs-build).
- PrimJS: `tools/primjs/out/Default/qjs`, built from `https://github.com/lynx-family/primjs` with `gn gen` and `ninja ... qjs_exe`, matching [`ahaoboy/primjs-build`](https://github.com/ahaoboy/primjs-build).

## Reference Config Check

`ahaoboy/js-engine-benchmark` delegates QuickJS and PrimJS installation to `ahaoboy/quickjs-build` and `ahaoboy/primjs-build`. The local build follows those referenced engine builds for the current `arm64` machine.

PrimJS performance-relevant GN args are explicitly set to the same values used by `ahaoboy/primjs-build` on arm64:

```gn
enable_quickjs_debugger = false
is_debug = false
target_cpu = "arm64"
enable_primjs_snapshot = true
enable_compatible_mm = true
enable_tracing_gc = true
enable_optimize_with_O2 = true
```

`enable_primjs_snapshot` is therefore enabled in the local PrimJS benchmark build. This macOS host has Command Line Tools without a full Xcode app selected, so the local GN file also pins the CLT SDK/toolchain paths (`use_xcode=false`, explicit `mac_sdk_path`, and related SDK path args). Those are build-environment overrides only; they are not PrimJS runtime-performance toggles.

PrimJS's CLI treats extra command-line tokens as more files, so the benchmark script generates `artifacts/primjs-runner.js` with `samples` and `scale` baked in.

## Wasm Build Config

The scalar WAMR artifact is built with:

```sh
RUSTFLAGS="-C target-cpu=generic -C target-feature=+bulk-memory,+bulk-memory-opt,+multivalue,+mutable-globals,+nontrapping-fptoint,+reference-types,+sign-ext,+simd128,+tail-call" cargo build --release --target wasm32-wasip1
wasm-opt -O3 --enable-sign-ext --enable-mutable-globals --enable-nontrapping-float-to-int --enable-bulk-memory --enable-bulk-memory-opt --enable-reference-types --enable-multivalue --enable-simd --enable-tail-call --enable-gc --enable-memory64 --enable-multimemory target/wasm32-wasip1/release/embedded-runtime-benchmarking.wasm -o artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
iwasm --interp artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
wasmi --compilation-mode eager artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
```

The threaded WAMR artifact is built from the same Rust package, with the copied threaded case set in `src/threaded_cases.rs`, and run with `--threads --workers 4`:

```sh
RUSTFLAGS="-C target-cpu=generic -C target-feature=+bulk-memory,+bulk-memory-opt,+multivalue,+mutable-globals,+nontrapping-fptoint,+reference-types,+sign-ext,+simd128,+tail-call --cfg wasip1_threads" cargo build --release --target wasm32-wasip1-threads
wasm-opt -O3 --enable-sign-ext --enable-mutable-globals --enable-nontrapping-float-to-int --enable-bulk-memory --enable-bulk-memory-opt --enable-reference-types --enable-multivalue --enable-simd --enable-tail-call --enable-gc --enable-memory64 --enable-multimemory --enable-threads target/wasm32-wasip1-threads/release/embedded-runtime-benchmarking.wasm -o artifacts/embedded-runtime-benchmarking.wasip1-threads.opt.wasm
iwasm --interp artifacts/embedded-runtime-benchmarking.wasip1-threads.opt.wasm --threads --workers 4
```

The Rust compile flags intentionally set `-C target-cpu=generic`, then enable the WAMR stable WebAssembly features that Rust/LLVM can emit for these targets: `bulk-memory, bulk-memory-opt, multivalue, mutable-globals, nontrapping-fptoint, reference-types, sign-ext, simd128, tail-call`. The threaded build uses `wasm32-wasip1-threads`, whose Rust target supplies the WASI threads shared-memory/atomics ABI and requires `bulk-memory`, `mutable-globals`, and `atomics`.

`iwasm --interp` selects interpreter execution. Use a WAMR build with `WAMR_BUILD_FAST_INTERP=1` enabled; WAMR documentation lists fast interpreter as the default build configuration and describes it as the optimized interpreter tier. `wasm32-wasip1-threads` additionally requires a WAMR build with `WAMR_BUILD_LIB_WASI_THREADS=1`.

The scalar WAMR runtime is built from WAMR 2.4.4 with these stable WAMR runtime proposals enabled: `bulk-memory, simd128, gc, memory64, multi-memory, reference-types, tail-call, shared-memory/threads, typed-function-references`. WAMR 2.4.4 reports these phase-4-or-newer proposals as unsupported and they are intentionally not emitted: `branch-hinting, custom-annotation-syntax, exception-handling, extended-constant-expressions, import/export-mutable-globals, js-string-builtins, relaxed-simd`.

The wasmi comparison uses `wasmi_cli 2.0.0-beta.2` installed with the default `wasi` feature and the additional `simd` feature. The benchmark runs with `--compilation-mode eager`, so wasmi translates the optimized module before entering `_start`; this keeps per-case timings focused on benchmark execution rather than first-touch translation.

The threaded WAMR comparison uses a second WAMR 2.4.4 fast-interpreter binary with the maximum stable runtime feature set that passed `wasm32-wasip1-threads` validation on this machine: `bulk-memory, simd128, memory64, multi-memory, reference-types, tail-call, shared-memory/threads`. Enabling `WAMR_BUILD_GC=1`/typed function references together with wasi-threads made the WAMR fast-interpreter run hang or exit 139 on this host, so the threaded runtime keeps GC off to preserve functional correctness. The Rust threaded artifact still uses `wasm32-wasip1-threads`; atomics/shared-memory ABI comes from the Rust target, and the benchmark selects the copied threaded cases with `--cfg wasip1_threads`.

The Wasmtime comparisons use the same optimized scalar `wasm32-wasip1` artifact as WAMR:

```sh
wasmtime run -C cache=n --target pulley64 artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
tools/wasmtime-pulley-tail-nightly/bin/wasmtime run -C cache=n --target pulley64 artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
wasmtime run -C cache=n -C compiler=cranelift artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
```

`wasmtime-pulley` selects Wasmtime's portable interpreter by using the Pulley target. `wasmtime-pulley-tail` uses a separately built Wasmtime 45.0.0 CLI with Pulley's nightly-only guaranteed tail-call loop enabled:

```sh
RUSTFLAGS="--cfg=pulley_tail_calls" CARGO_TARGET_DIR=target/wasmtime-pulley-tail-nightly cargo +nightly install wasmtime-cli --version 45.0.0 --root tools/wasmtime-pulley-tail-nightly --features pulley --locked --force
```

The stable-Rust `--cfg=pulley_assume_llvm_makes_tail_calls` Pulley path was also tested on this macOS arm64 host and crashed with `Bus error: 10`, so it is not included in the results. `wasmtime-jit` selects Cranelift explicitly. All Wasmtime runs disable Wasmtime's persistent compilation cache with `-C cache=n` so results are not affected by a previous command-line cache entry.

The Wasmtime threaded comparisons use the optimized `wasm32-wasip1-threads` artifact:

```sh
wasmtime run -C cache=n --target pulley64 -S threads=y -W threads=y -W shared-memory=y artifacts/embedded-runtime-benchmarking.wasip1-threads.opt.wasm --threads --workers 4
tools/wasmtime-pulley-tail-nightly/bin/wasmtime run -C cache=n --target pulley64 -S threads=y -W threads=y -W shared-memory=y artifacts/embedded-runtime-benchmarking.wasip1-threads.opt.wasm --threads --workers 4
wasmtime run -C cache=n -C compiler=cranelift -S threads=y -W threads=y -W shared-memory=y artifacts/embedded-runtime-benchmarking.wasip1-threads.opt.wasm --threads --workers 4
```

Wasmtime 45.0.0 on this host reports `wasm_threads` as unsupported for both Pulley compiler configurations, so `wasmtime-pulley-threads` and `wasmtime-pulley-tail-threads` are included in the status table but have no timing samples. `wasmtime-jit-threads` runs successfully with WASI threads enabled.

## Case Rewrite Policy

- The Rust code uses edition 2024 and the pinned `1.95.0` toolchain in `rust-toolchain.toml`.
- The Rust cases are not line-by-line ports of the JavaScript file. They keep the same broad workload names while using Rust-friendly representations: fixed arrays for small static networks, arena/index-style constraint plans for DeltaBlue, precomputed ray data for RayTrace, byte slices for the ASCII DNA workload, and reusable buffers for grid simulation.
- Standard-library containers are used where appropriate: `VecDeque` for Richards scheduler queues, compact `Vec` state sets for the small Earley chart, and `BTreeMap` for the ordered-map workload inspired by Splay.
- The threaded Rust cases use `std::thread` with deterministic per-worker checksum reduction. Each worker runs a disjoint iteration chunk; no shared mutable benchmark state is used.
- No external crates are required, keeping native and WASI builds reproducible in an empty repository.

## References

- [ahaoboy/js-engine-benchmark](https://github.com/ahaoboy/js-engine-benchmark)
- [ahaoboy V8 v7 case files](https://github.com/ahaoboy/js-engine-benchmark/tree/main/v8-v7)
- [ahaoboy V8 v7 score harness](https://github.com/ahaoboy/js-engine-benchmark/blob/main/v8-v7/base.js)
- [ahaoboy V8 v7 runner](https://github.com/ahaoboy/js-engine-benchmark/blob/main/v8-v7/run.js)
- [V8 benchmark documentation](https://v8.dev/docs/benchmarks)
- [WAMR running modes](https://bytecodealliance.github.io/wamr.dev/blog/introduction-to-wamr-running-modes/)
- [WAMR README](https://github.com/bytecodealliance/wasm-micro-runtime)
- [WAMR WebAssembly proposal stability](https://github.com/bytecodealliance/wasm-micro-runtime/blob/main/doc/stability_wasm_proposals.md)
- [wasmi](https://github.com/wasmi-labs/wasmi)
- [Wasmtime Pulley documentation](https://docs.wasmtime.dev/examples-pulley.html)
- [Wasmtime CLI options](https://docs.wasmtime.dev/cli-options.html)
- [Rust wasm32-wasip1-threads target](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1-threads.html)
