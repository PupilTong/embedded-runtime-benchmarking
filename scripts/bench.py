#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
import os
import platform
import shutil
import statistics
import subprocess
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RESULTS_DIR = ROOT / "results"
ARTIFACTS_DIR = ROOT / "artifacts"
README = ROOT / "readme.md"
JS_RUNNER = ROOT / "benchmarks" / "js" / "runner.js"
WASM_TARGET = "wasm32-wasip1"
WASM_THREADS_TARGET = "wasm32-wasip1-threads"
WASM_RELEASE = ROOT / "target" / WASM_TARGET / "release" / "embedded-runtime-benchmarking.wasm"
WASM_THREADS_RELEASE = ROOT / "target" / WASM_THREADS_TARGET / "release" / "embedded-runtime-benchmarking.wasm"
WASM_OPT = ARTIFACTS_DIR / "embedded-runtime-benchmarking.wasip1.opt.wasm"
WASM_THREADS_OPT = ARTIFACTS_DIR / "embedded-runtime-benchmarking.wasip1-threads.opt.wasm"
WAMR_DARWIN_DIR = ROOT / "tools" / "wasm-micro-runtime" / "product-mini" / "platforms" / "darwin"
WAMR_FULL_IWASM = WAMR_DARWIN_DIR / "build" / "iwasm"
WAMR_THREADS_IWASM = WAMR_DARWIN_DIR / "build-threads-mem" / "iwasm"

WAMR_STABLE_RUST_FEATURES = [
    "bulk-memory",
    "bulk-memory-opt",
    "multivalue",
    "mutable-globals",
    "nontrapping-fptoint",
    "reference-types",
    "sign-ext",
    "simd128",
    "tail-call",
]
WAMR_STABLE_WASM_OPT_FLAGS = [
    "--enable-sign-ext",
    "--enable-mutable-globals",
    "--enable-nontrapping-float-to-int",
    "--enable-bulk-memory",
    "--enable-bulk-memory-opt",
    "--enable-reference-types",
    "--enable-multivalue",
    "--enable-simd",
    "--enable-tail-call",
    "--enable-gc",
    "--enable-memory64",
    "--enable-multimemory",
]
WAMR_SCALAR_RUNTIME_FEATURES = [
    "bulk-memory",
    "simd128",
    "gc",
    "memory64",
    "multi-memory",
    "reference-types",
    "tail-call",
    "shared-memory/threads",
    "typed-function-references",
]
WAMR_THREADS_RUNTIME_FEATURES = [
    "bulk-memory",
    "simd128",
    "memory64",
    "multi-memory",
    "reference-types",
    "tail-call",
    "shared-memory/threads",
]
WAMR_UNSUPPORTED_STABLE_FEATURES = [
    "branch-hinting",
    "custom-annotation-syntax",
    "exception-handling",
    "extended-constant-expressions",
    "import/export-mutable-globals",
    "js-string-builtins",
    "relaxed-simd",
]
V8_V7_REFERENCE_SCORES = {
    "Richards": 35302,
    "DeltaBlue": 66118,
    "Crypto": 266181,
    "RayTrace": 739989,
    "EarleyBoyer": 666463,
    "RegExp": 910985,
    "Splay": 81491,
    "NavierStokes": 1484000,
}


def run_command(
    command: list[str],
    *,
    cwd: Path = ROOT,
    timeout: int | None = None,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    process_env = os.environ.copy()
    if env:
        process_env.update(env)
    return subprocess.run(
        command,
        cwd=str(cwd),
        env=process_env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )


def compact_error(message: str, limit: int = 800) -> str:
    lines = [line.rstrip() for line in message.splitlines() if line.strip()]
    compact = "\n".join(lines)
    return compact if len(compact) <= limit else compact[:limit].rstrip() + "\n..."


def command_text(command: list[str], *, timeout: int = 20) -> str:
    try:
        completed = run_command(command, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return str(exc)
    output = (completed.stdout or completed.stderr).strip()
    return output.splitlines()[0] if output else f"exit {completed.returncode}"


def file_size(path: str | None) -> int:
    if not path:
        return 0
    try:
        return Path(path).stat().st_size
    except OSError:
        return 0


def dependency_paths(binary: str | None) -> list[str]:
    if not binary:
        return []
    system = platform.system()
    try:
        if system == "Darwin":
            completed = run_command(["otool", "-L", binary], timeout=20)
            if completed.returncode != 0:
                return []
            return [
                line.strip().split(" ")[0]
                for line in completed.stdout.splitlines()[1:]
                if line.strip()
            ]
        if system == "Linux":
            completed = run_command(["ldd", binary], timeout=20)
            if completed.returncode != 0:
                return []
            paths = []
            for line in completed.stdout.splitlines():
                path = line.split("=>")[-1].strip().split(" (")[0]
                if path and not path.startswith(("linux-", "/lib/", "/lib64/")):
                    paths.append(path)
            return paths
    except OSError:
        return []
    return []


def binary_size(binary: str | None) -> dict[str, int]:
    exe_size = file_size(binary)
    dll_size = sum(file_size(path) for path in dependency_paths(binary))
    return {
        "exe_size": exe_size,
        "dll_size": dll_size,
        "total_size": exe_size + dll_size,
    }


def quickjs_version(binary: str) -> str:
    try:
        completed = run_command([binary, "--version"], timeout=20)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return str(exc)
    output = "\n".join(part for part in [completed.stdout, completed.stderr] if part).strip()
    for line in output.splitlines():
        if "version" in line.lower():
            return line.strip()
    return output.splitlines()[0] if output else f"exit {completed.returncode}"


def primjs_version(binary: str) -> str:
    path = Path(binary).resolve()
    for parent in path.parents:
        version_file = parent / "PRIMJS_VERSION"
        if version_file.exists():
            return f"PrimJS {version_file.read_text(encoding='utf-8').strip()}"
    return "PrimJS"


def find_tool(env_name: str, candidates: list[str]) -> str | None:
    configured = os.environ.get(env_name)
    if configured:
        resolved = shutil.which(configured)
        return resolved or configured
    for candidate in candidates:
        resolved = shutil.which(candidate)
        if resolved:
            return resolved
    return None


def parse_json_lines(output: str, runtime: str) -> list[dict]:
    samples: list[dict] = []
    for line in output.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("event") == "sample":
            payload["runtime"] = runtime
            samples.append(payload)
    return samples


def summarize(samples: list[dict]) -> dict[str, dict[str, float]]:
    grouped: dict[str, dict[str, list[float]]] = {}
    for sample in samples:
        runtime = sample["runtime"]
        case = sample["case"]
        grouped.setdefault(runtime, {}).setdefault(case, []).append(float(sample["elapsed_ms"]))

    summary: dict[str, dict[str, float]] = {}
    for runtime, cases in grouped.items():
        summary[runtime] = {}
        for case, values in cases.items():
            summary[runtime][case] = statistics.median(values)
    return summary


def run_json_benchmark(runtime: str, command: list[str], timeout: int) -> tuple[list[dict], str | None]:
    try:
        completed = run_command(command, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired) as exc:
        return [], str(exc)

    if completed.returncode != 0:
        detail = "\n".join(part for part in [completed.stdout, completed.stderr] if part).strip()
        return [], detail or f"{runtime} exited with code {completed.returncode}"

    samples = parse_json_lines(completed.stdout, runtime)
    if not samples:
        return [], f"{runtime} produced no JSON sample lines"
    return samples, None


def primjs_runner(samples: int, scale: int) -> Path:
    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    baked = ARTIFACTS_DIR / "primjs-runner.js"
    source = JS_RUNNER.read_text(encoding="utf-8")
    source = source.replace(
        "var options = { samples: 5, scale: 1, caseFilter: null };",
        f"var options = {{ samples: {samples}, scale: {scale}, caseFilter: null }};",
    )
    baked.write_text(source, encoding="utf-8")
    return baked


def rustflags(features: list[str], *, cfgs: list[str] | None = None) -> str:
    enabled = ",".join(f"+{feature}" for feature in features)
    cfg_flags = " ".join(f"--cfg {cfg}" for cfg in cfgs or [])
    return f"-C target-cpu=generic -C target-feature={enabled} {cfg_flags}".strip()


def build_wasm(
    cargo_bin: str,
    target: str,
    artifact: Path,
    features: list[str],
    *,
    cfgs: list[str] | None = None,
) -> str | None:
    completed = run_command(
        [cargo_bin, "build", "--release", "--target", target],
        env={"RUSTFLAGS": rustflags(features, cfgs=cfgs)},
        timeout=600,
    )
    if completed.returncode != 0:
        return (completed.stderr or completed.stdout).strip() or "cargo build failed"
    if not artifact.exists():
        return f"missing wasm artifact: {artifact}"
    return None


def optimize_wasm(wasm_opt_bin: str, source: Path, output: Path, *, threads: bool) -> str | None:
    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    feature_flags = WAMR_STABLE_WASM_OPT_FLAGS + (["--enable-threads"] if threads else [])
    completed = run_command(
        [
            wasm_opt_bin,
            "-O3",
            *feature_flags,
            str(source),
            "-o",
            str(output),
        ],
        timeout=180,
    )
    if completed.returncode != 0:
        return compact_error((completed.stderr or completed.stdout).strip() or "wasm-opt failed")
    if not output.exists():
        return f"missing optimized wasm artifact: {output}"
    return None


def runtime_status(name: str, binary: str | None, version: str | None, status: str) -> dict:
    return {
        "name": name,
        "binary": binary or "",
        "version": version or "",
        "status": status,
    }


def markdown_table(headers: list[str], rows: list[list[str]]) -> str:
    lines = [
        "| " + " | ".join(headers) + " |",
        "| " + " | ".join("---" for _ in headers) + " |",
    ]
    for row in rows:
        lines.append("| " + " | ".join(row) + " |")
    return "\n".join(lines)


def format_ms(value: float | None) -> str:
    return "-" if value is None else f"{value:.3f}"


def fixed_one(value: float) -> str:
    fixed = f"{value:.1f}"
    return fixed[:-2] if fixed.endswith(".0") else fixed


def human_size(value: int | float | None) -> str:
    if not value:
        return "0"
    kib = float(value) / 1024.0
    if kib < 1024.0:
        return f"{fixed_one(kib)}K"
    mib = kib / 1024.0
    if mib < 1024.0:
        return f"{fixed_one(mib)}M"
    return f"{fixed_one(mib / 1024.0)}G"


def geometric_mean(values: list[float]) -> float | None:
    if not values or any(value <= 0.0 for value in values):
        return None
    return math.exp(sum(math.log(value) for value in values) / len(values))


def v8_v7_case_score(reference: float, elapsed_ms: float | None) -> float | None:
    if elapsed_ms is None or elapsed_ms <= 0.0:
        return None
    return 100.0 * reference / (elapsed_ms * 1000.0)


def format_v8_v7_score(value: float | None) -> str:
    if value is None:
        return "-"
    return f"{value:.0f}" if value > 100.0 else f"{value:.3g}"


def score_per_mb(score: float | None, total_size: int) -> int | None:
    if score is None or total_size <= 0:
        return None
    return math.floor(score / total_size * 1024 * 1024)


def format_int(value: int | None) -> str:
    return "-" if value is None else str(value)


def generate_readme(report: dict) -> str:
    summary = summarize(report["samples"])
    cases = [
        "Richards",
        "DeltaBlue",
        "Crypto",
        "RayTrace",
        "EarleyBoyer",
        "RegExp",
        "Splay",
        "NavierStokes",
    ]
    runtimes = ["quickjs", "primjs", "wamr-fast-interp", "wamr-fast-interp-threads"]

    result_rows: list[list[str]] = []
    for case in cases:
        values = {runtime: summary.get(runtime, {}).get(case) for runtime in runtimes}
        present = {runtime: value for runtime, value in values.items() if value is not None}
        fastest = min(present, key=present.get) if present else "-"
        result_rows.append([case, *(format_ms(values[runtime]) for runtime in runtimes), fastest])

    tool_by_runtime = {item["name"]: item for item in report["tools"]}
    size_by_runtime = report.get("sizes", {})

    score_rows: list[dict] = []
    for runtime in runtimes:
        case_scores = [
            v8_v7_case_score(V8_V7_REFERENCE_SCORES[case], summary.get(runtime, {}).get(case))
            for case in cases
        ]
        valid_scores = [score for score in case_scores if score is not None]
        total_score = geometric_mean(valid_scores) if len(valid_scores) == len(cases) else None
        sort_score = total_score if total_score is not None else -1.0
        sizes = size_by_runtime.get(runtime, {})
        score_rows.append(
            {
                "runtime": runtime,
                "sort_score": sort_score,
                "version": tool_by_runtime.get(runtime, {}).get("version", ""),
                "sizes": sizes,
                "case_scores": case_scores,
                "total_score": total_score,
                "score_per_mb": score_per_mb(total_score, int(sizes.get("total_size", 0))),
            }
        )
    score_rows.sort(key=lambda row: row["sort_score"], reverse=True)

    score_metric_rows = [
        ["Version", *(row["version"] or "-" for row in score_rows)],
        ["Total size", *(human_size(row["sizes"].get("total_size", 0)) for row in score_rows)],
        ["Exe size", *(human_size(row["sizes"].get("exe_size", 0)) for row in score_rows)],
        ["Dll size", *(human_size(row["sizes"].get("dll_size", 0)) for row in score_rows)],
    ]
    for index, case in enumerate(cases):
        score_metric_rows.append(
            [case, *(format_v8_v7_score(row["case_scores"][index]) for row in score_rows)]
        )
    score_metric_rows.extend(
        [
            ["Score", *(format_v8_v7_score(row["total_score"]) for row in score_rows)],
            ["Score/MB", *(format_int(row["score_per_mb"]) for row in score_rows)],
        ]
    )

    status_rows = [
        [item["name"], item["binary"] or "-", item["version"] or "-", item["status"]]
        for item in report["tools"]
    ]

    notes = report["notes"]
    note_block = "\n".join(f"- {note}" for note in notes) if notes else "- none"

    valid_samples = len(report["samples"])
    generated = report["generated_at"]
    arch = report["host"]["arch"]
    os_name = report["host"]["os"]

    return f"""# Embedded Runtime Benchmarking

This repository benchmarks the V8 v7-style workload shape used by [ahaoboy/js-engine-benchmark](https://github.com/ahaoboy/js-engine-benchmark): Richards, DeltaBlue, Crypto, RayTrace, EarleyBoyer, RegExp, Splay, and NavierStokes. The JavaScript engines run the JS workload in `benchmarks/js/runner.js`; WAMR runs an idiomatic Rust 2024 rewrite compiled to WASI preview1 wasm.

## Current Machine

- Generated at: `{generated}`
- Host: `{os_name} {arch}`
- Rust: `{report["host"]["rustc"]}`
- Cargo: `{report["host"]["cargo"]}`
- WASI targets: `wasm32-wasip1` and `wasm32-wasip1-threads`
- Samples per case: `{report["options"]["samples"]}`
- Scale: `{report["options"]["scale"]}`
- Thread workers: `{report["options"]["workers"]}`
- Valid sample rows: `{valid_samples}`

## Runtime Status

{markdown_table(["Runtime", "Binary", "Version", "Status"], status_rows)}

## Results

Median elapsed time in milliseconds. Lower is better.

{markdown_table(["Case", "quickjs", "primjs", "wamr-fast-interp", "wamr-fast-interp-threads", "Fastest"], result_rows)}

## V8 V7 Style Score

Higher is better. This table follows the score shape from `ahaoboy/js-engine-benchmark`'s V8 v7 harness: each case score is `100 * reference / median_us`, then `Score` is the geometric mean of the case scores. The reference constants are the `BenchmarkSuite(..., reference, ...)` values from the upstream V8 v7 case files. `Score/MB` follows upstream as `Score / Total size in MiB`, where `Total size = Exe size + Dll size` for the runtime binary. This repository uses median full-sample timings rather than the upstream one-second harness average, so the scores are intended for comparing rows in this report, not as official V8/Octane scores.

{markdown_table(["Metric", *(row["runtime"] for row in score_rows)], score_metric_rows)}

## Notes

{note_block}

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
RUSTFLAGS="{report["wasm"]["scalar_rustflags"]}" cargo build --release --target wasm32-wasip1
wasm-opt -O3 {" ".join(report["wasm"]["wasm_opt_flags"])} target/wasm32-wasip1/release/embedded-runtime-benchmarking.wasm -o artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
iwasm --interp artifacts/embedded-runtime-benchmarking.wasip1.opt.wasm
```

The threaded WAMR artifact is built from the same Rust package, with the copied threaded case set in `src/threaded_cases.rs`, and run with `--threads --workers {report["options"]["workers"]}`:

```sh
RUSTFLAGS="{report["wasm"]["threads_rustflags"]}" cargo build --release --target wasm32-wasip1-threads
wasm-opt -O3 {" ".join(report["wasm"]["wasm_opt_flags"] + ["--enable-threads"])} target/wasm32-wasip1-threads/release/embedded-runtime-benchmarking.wasm -o artifacts/embedded-runtime-benchmarking.wasip1-threads.opt.wasm
iwasm --interp artifacts/embedded-runtime-benchmarking.wasip1-threads.opt.wasm --threads --workers {report["options"]["workers"]}
```

The Rust compile flags intentionally set `-C target-cpu=generic`, then enable the WAMR stable WebAssembly features that Rust/LLVM can emit for these targets: `{", ".join(report["wasm"]["scalar_features"])}`. The threaded build uses `wasm32-wasip1-threads`, whose Rust target supplies the WASI threads shared-memory/atomics ABI and requires `bulk-memory`, `mutable-globals`, and `atomics`.

`iwasm --interp` selects interpreter execution. Use a WAMR build with `WAMR_BUILD_FAST_INTERP=1` enabled; WAMR documentation lists fast interpreter as the default build configuration and describes it as the optimized interpreter tier. `wasm32-wasip1-threads` additionally requires a WAMR build with `WAMR_BUILD_LIB_WASI_THREADS=1`.

The scalar WAMR runtime is built from WAMR 2.4.4 with these stable WAMR runtime proposals enabled: `{", ".join(report["wasm"]["scalar_runtime_features"])}`. WAMR 2.4.4 reports these phase-4-or-newer proposals as unsupported and they are intentionally not emitted: `{", ".join(report["wasm"]["unsupported_wamr_features"])}`.

The threaded WAMR comparison uses a second WAMR 2.4.4 fast-interpreter binary with the maximum stable runtime feature set that passed `wasm32-wasip1-threads` validation on this machine: `{", ".join(report["wasm"]["threads_runtime_features"])}`. Enabling `WAMR_BUILD_GC=1`/typed function references together with wasi-threads made the WAMR fast-interpreter run hang or exit 139 on this host, so the threaded runtime keeps GC off to preserve functional correctness. The Rust threaded artifact still uses `wasm32-wasip1-threads`; atomics/shared-memory ABI comes from the Rust target, and the benchmark selects the copied threaded cases with `--cfg wasip1_threads`.

## Case Rewrite Policy

- The Rust code uses edition 2024 and the pinned `{report["host"]["rustc"].split()[1]}` toolchain in `rust-toolchain.toml`.
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
- [Rust wasm32-wasip1-threads target](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1-threads.html)
"""


def main() -> int:
    parser = argparse.ArgumentParser(description="Run embedded runtime benchmarks and update readme.md")
    parser.add_argument("--samples", type=int, default=int(os.environ.get("BENCH_SAMPLES", "5")))
    parser.add_argument("--scale", type=int, default=int(os.environ.get("BENCH_SCALE", "1")))
    parser.add_argument("--workers", type=int, default=int(os.environ.get("BENCH_WORKERS", "4")))
    parser.add_argument("--timeout", type=int, default=int(os.environ.get("BENCH_TIMEOUT", "900")))
    parser.add_argument("--skip-build", action="store_true")
    args = parser.parse_args()

    samples = max(1, args.samples)
    scale = max(1, args.scale)
    workers = max(1, args.workers)

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    tools: list[dict] = []
    notes: list[str] = []
    all_samples: list[dict] = []

    quickjs = find_tool(
        "QUICKJS_BIN",
        [str(ROOT / "tools" / "quickjs" / "qjs"), str(ROOT / "tools" / "quickjs-2025-09-13" / "qjs"), "qjs", "quickjs"],
    )
    primjs = find_tool(
        "PRIMJS_BIN",
        [str(ROOT / "tools" / "primjs" / "out" / "Default" / "qjs"), "primjs"],
    )
    iwasm = find_tool(
        "IWASM_BIN",
        [str(WAMR_FULL_IWASM), "iwasm"],
    )
    iwasm_threads_candidates = [str(WAMR_THREADS_IWASM)]
    if iwasm:
        iwasm_threads_candidates.append(iwasm)
    iwasm_threads_candidates.append("iwasm")
    iwasm_threads = find_tool("IWASM_THREADS_BIN", iwasm_threads_candidates)
    wasm_opt = find_tool("WASM_OPT_BIN", ["wasm-opt"])
    cargo = find_tool("CARGO_BIN", ["cargo"])

    if quickjs:
        version = quickjs_version(quickjs)
        js_samples, error = run_json_benchmark(
            "quickjs",
            [quickjs, str(JS_RUNNER), "--samples", str(samples), "--scale", str(scale)],
            args.timeout,
        )
        all_samples.extend(js_samples)
        tools.append(runtime_status("quickjs", quickjs, version, "ok" if not error else f"failed: {error}"))
    else:
        tools.append(runtime_status("quickjs", None, None, "missing; set QUICKJS_BIN or install qjs"))
        notes.append("quickjs was not benchmarked because `qjs`/`quickjs` was not found in PATH.")

    if primjs:
        version = primjs_version(primjs)
        primjs_js = primjs_runner(samples, scale)
        js_samples, error = run_json_benchmark(
            "primjs",
            [primjs, str(primjs_js)],
            args.timeout,
        )
        all_samples.extend(js_samples)
        tools.append(runtime_status("primjs", primjs, version, "ok" if not error else f"failed: {error}"))
    else:
        tools.append(runtime_status("primjs", None, None, "missing; set PRIMJS_BIN or install primjs"))
        notes.append("primjs was not benchmarked because `primjs` was not found in PATH.")

    scalar_features = WAMR_STABLE_RUST_FEATURES
    threads_features = WAMR_STABLE_RUST_FEATURES
    wasm_ready = False
    wasm_threads_ready = False
    if cargo and not args.skip_build:
        build_error = build_wasm(cargo, WASM_TARGET, WASM_RELEASE, scalar_features)
        if build_error:
            notes.append(f"WASI build failed: {build_error}")
        elif wasm_opt:
            optimize_error = optimize_wasm(wasm_opt, WASM_RELEASE, WASM_OPT, threads=False)
            if optimize_error:
                notes.append(f"`wasm-opt -O3` failed: {optimize_error}")
            else:
                wasm_ready = True
        else:
            notes.append("WAMR was not benchmarked because `wasm-opt` was not found; the required optimized wasm artifact was not produced.")

        build_threads_error = build_wasm(
            cargo,
            WASM_THREADS_TARGET,
            WASM_THREADS_RELEASE,
            threads_features,
            cfgs=["wasip1_threads"],
        )
        if build_threads_error:
            notes.append(f"WASI threads build failed: {build_threads_error}")
        elif wasm_opt:
            optimize_threads_error = optimize_wasm(wasm_opt, WASM_THREADS_RELEASE, WASM_THREADS_OPT, threads=True)
            if optimize_threads_error:
                notes.append(f"`wasm-opt -O3` for WASI threads failed: {optimize_threads_error}")
            else:
                wasm_threads_ready = True
    elif not cargo:
        notes.append("WASI build was skipped because `cargo` was not found.")
    else:
        wasm_ready = WASM_OPT.exists()
        wasm_threads_ready = WASM_THREADS_OPT.exists()

    if iwasm and wasm_ready:
        version = command_text([iwasm, "--version"])
        wasm_samples, error = run_json_benchmark(
            "wamr-fast-interp",
            [iwasm, "--interp", str(WASM_OPT), "--samples", str(samples), "--scale", str(scale)],
            args.timeout,
        )
        all_samples.extend(wasm_samples)
        tools.append(
            runtime_status(
                "wamr-fast-interp",
                iwasm,
                version,
                "ok" if not error else f"failed: {compact_error(error)}",
            )
        )
    elif iwasm:
        version = command_text([iwasm, "--version"])
        tools.append(runtime_status("wamr-fast-interp", iwasm, version, "skipped; optimized wasm artifact unavailable"))
    else:
        tools.append(runtime_status("wamr-fast-interp", None, None, "missing; set IWASM_BIN or install iwasm"))
        notes.append("WAMR was not benchmarked because `iwasm` was not found in PATH.")

    if iwasm_threads and wasm_threads_ready:
        version = command_text([iwasm_threads, "--version"])
        wasm_thread_samples, error = run_json_benchmark(
            "wamr-fast-interp-threads",
            [
                iwasm_threads,
                "--interp",
                str(WASM_THREADS_OPT),
                "--threads",
                "--workers",
                str(workers),
                "--samples",
                str(samples),
                "--scale",
                str(scale),
            ],
            args.timeout,
        )
        all_samples.extend(wasm_thread_samples)
        tools.append(
            runtime_status(
                "wamr-fast-interp-threads",
                iwasm_threads,
                version,
                "ok" if not error else f"failed: {compact_error(error)}",
            )
        )
    elif iwasm_threads:
        version = command_text([iwasm_threads, "--version"])
        tools.append(
            runtime_status(
                "wamr-fast-interp-threads",
                iwasm_threads,
                version,
                "skipped; optimized wasm32-wasip1-threads artifact unavailable",
            )
        )
    else:
        tools.append(
            runtime_status(
                "wamr-fast-interp-threads",
                None,
                None,
                "missing; set IWASM_THREADS_BIN or install a wasi-threads-enabled iwasm",
            )
        )

    sizes = {item["name"]: binary_size(item["binary"]) for item in tools}

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "host": {
            "os": platform.system(),
            "arch": platform.machine(),
            "rustc": command_text(["rustc", "--version"]),
            "cargo": command_text(["cargo", "--version"]),
        },
        "options": {"samples": samples, "scale": scale, "workers": workers},
        "wasm": {
            "scalar_target": WASM_TARGET,
            "threads_target": WASM_THREADS_TARGET,
            "scalar_features": scalar_features,
            "threads_features": threads_features,
            "scalar_rustflags": rustflags(scalar_features),
            "threads_rustflags": rustflags(threads_features, cfgs=["wasip1_threads"]),
            "wasm_opt_flags": WAMR_STABLE_WASM_OPT_FLAGS,
            "scalar_runtime_features": WAMR_SCALAR_RUNTIME_FEATURES,
            "threads_runtime_features": WAMR_THREADS_RUNTIME_FEATURES,
            "unsupported_wamr_features": WAMR_UNSUPPORTED_STABLE_FEATURES,
        },
        "score": {
            "algorithm": "v8-v7",
            "case_formula": "100 * reference / median_us",
            "total_formula": "geometric_mean(case_scores)",
            "score_per_mb_formula": "floor(score / total_size_bytes * 1024 * 1024)",
            "references": V8_V7_REFERENCE_SCORES,
        },
        "sizes": sizes,
        "tools": tools,
        "samples": all_samples,
        "notes": notes,
    }

    result_file = RESULTS_DIR / "current.json"
    result_file.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    README.write_text(generate_readme(report), encoding="utf-8")
    print(f"wrote {result_file}")
    print(f"wrote {README}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
