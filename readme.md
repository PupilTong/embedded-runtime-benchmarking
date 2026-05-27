# Embedded Runtime Benchmarking

This repository benchmarks the V8 v7-style workload shape used by [ahaoboy/js-engine-benchmark](https://github.com/ahaoboy/js-engine-benchmark): Richards, DeltaBlue, Crypto, RayTrace, EarleyBoyer, RegExp, Splay, and NavierStokes. The JavaScript engines run the JS workload in `benchmarks/js/runner.js`; WAMR runs an idiomatic Rust 2024 rewrite compiled to WASI preview1 wasm.

## Current Machine

- Generated at: `2026-05-27T07:01:54+00:00`
- Host: `Darwin arm64`
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- WASI targets: `wasm32-wasip1` and `wasm32-wasip1-threads`
- Samples per case: `5`
- Scale: `1`
- Thread workers: `4`
- Valid sample rows: `160`

## Runtime Status

| Runtime | Binary | Version | Status |
| --- | --- | --- | --- |
| quickjs | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/quickjs/qjs | QuickJS version 2025-09-13 | ok |
| primjs | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/primjs/out/Default/qjs | PrimJS 2.11.1-rc.1 | ok |
| wamr-fast-interp | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/wasm-micro-runtime/product-mini/platforms/darwin/build/iwasm | iwasm 2.4.4 | ok |
| wamr-fast-interp-threads | /Users/bytedance/Documents/embedded-runtime-benchmarking/tools/wasm-micro-runtime/product-mini/platforms/darwin/build-threads-mem/iwasm | iwasm 2.4.4 | ok |

## Results

Median elapsed time in milliseconds. Lower is better.

| Case | quickjs | primjs | wamr-fast-interp | wamr-fast-interp-threads | Fastest |
| --- | --- | --- | --- | --- | --- |
| Richards | 23.000 | 34.000 | 5.001 | 1.526 | wamr-fast-interp-threads |
| DeltaBlue | 16.000 | 16.000 | 84.941 | 21.986 | quickjs |
| Crypto | 527.000 | 840.000 | 19.682 | 5.508 | wamr-fast-interp-threads |
| RayTrace | 34.000 | 42.000 | 59.583 | 14.153 | wamr-fast-interp-threads |
| EarleyBoyer | 15.000 | 24.000 | 9.515 | 18.820 | wamr-fast-interp |
| RegExp | 85.000 | 113.000 | 137.066 | 33.674 | wamr-fast-interp-threads |
| Splay | 727.000 | 1386.000 | 6.172 | 3.346 | wamr-fast-interp-threads |
| NavierStokes | 38.000 | 44.000 | 14.866 | 3.381 | wamr-fast-interp-threads |

## Notes

- none

## How To Run

```sh
python3 scripts/bench.py --samples 5 --scale 1
```

Runtime binaries can be overridden with environment variables:

```sh
QUICKJS_BIN=/path/to/qjs PRIMJS_BIN=/path/to/primjs IWASM_BIN=/path/to/iwasm IWASM_THREADS_BIN=/path/to/iwasm WASM_OPT_BIN=/path/to/wasm-opt python3 scripts/bench.py
```

On macOS, Homebrew can provide the external optimizer/runtime tools:

```sh
brew install binaryen wasm-micro-runtime python@3.11
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

The threaded WAMR comparison uses a second WAMR 2.4.4 fast-interpreter binary with the maximum stable runtime feature set that passed `wasm32-wasip1-threads` validation on this machine: `bulk-memory, simd128, memory64, multi-memory, reference-types, tail-call, shared-memory/threads`. Enabling `WAMR_BUILD_GC=1`/typed function references together with wasi-threads made the WAMR fast-interpreter run hang or exit 139 on this host, so the threaded runtime keeps GC off to preserve functional correctness. The Rust threaded artifact still uses `wasm32-wasip1-threads`; atomics/shared-memory ABI comes from the Rust target, and the benchmark selects the copied threaded cases with `--cfg wasip1_threads`.

## Case Rewrite Policy

- The Rust code uses edition 2024 and the pinned `1.95.0` toolchain in `rust-toolchain.toml`.
- Standard-library containers are used where appropriate: `VecDeque` for scheduler queues, `BTreeSet` for Earley chart states, and `BTreeMap` for the ordered-map workload inspired by Splay.
- The threaded Rust cases use `std::thread` with deterministic per-worker checksum reduction. Each worker runs a disjoint iteration chunk; no shared mutable benchmark state is used.
- No external crates are required, keeping native and WASI builds reproducible in an empty repository.

## References

- [ahaoboy/js-engine-benchmark](https://github.com/ahaoboy/js-engine-benchmark)
- [V8 benchmark documentation](https://v8.dev/docs/benchmarks)
- [WAMR running modes](https://bytecodealliance.github.io/wamr.dev/blog/introduction-to-wamr-running-modes/)
- [WAMR README](https://github.com/bytecodealliance/wasm-micro-runtime)
- [WAMR WebAssembly proposal stability](https://github.com/bytecodealliance/wasm-micro-runtime/blob/main/doc/stability_wasm_proposals.md)
- [Rust wasm32-wasip1-threads target](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1-threads.html)
