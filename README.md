# kenken-explorer

[![CI](https://github.com/wpm/kenken-explorer/actions/workflows/ci.yml/badge.svg)](https://github.com/wpm/kenken-explorer/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/wpm/kenken-explorer/graph/badge.svg)](https://codecov.io/gh/wpm/kenken-explorer)

A command-line research tool for empirically studying [KenKen](https://www.kenken.com) puzzles, built on the [`kenken`](https://github.com/wpm/KenKen) library.

## Overview

kenken-explorer runs controlled experiments over large batches of randomly generated KenKen puzzles and reports results as JSON. Each subcommand is a self-contained experiment; results are one JSON object per run on stdout so that back-to-back invocations form a [JSON Lines](https://jsonlines.org) stream that loads directly into `jq`, pandas, or any other analysis tool.

## Building

```bash
cargo build --release
```

The binary is `target/release/kenken-explorer`.

## Global flags

| Flag | Description |
|------|-------------|
| `-c, --config FILE` | Path to a TOML config file. CLI flags take precedence over file values. |
| `-t, --threads N` | Worker thread count. Defaults to the number of logical cores. |

## Subcommands

### `generate`

Generate a single n×n puzzle and print its solution count and uniqueness. Useful for spot-checking the generator and verifying that a given seed produces a solvable puzzle.

| Flag | Description |
|------|-------------|
| `--n N` | Puzzle size (1–9). Default: 4. |
| `--seed N` | RNG seed. Default: 0. |
| `--op-policy POLICY` | Operator selection policy. Default: `default`. |

```bash
kenken-explorer generate --n 4 --seed 42
```

Output:

```json
{"experiment":"generate","config":{"n":4,"seed":42,"op_policy":"default","size_distribution":{"type":"uniform","min":1,"max":4}},"result":{"uniqueness":"multiple","solutions":2}}
```

### `histogram`

The main experiment. Generates `trials` independent n×n puzzles in parallel and counts solutions for each, up to `--max-solutions`. Reports a histogram of solution counts together with `unique_count`, `trials`, and `unique_rate` for downstream confidence-interval computation.

| Flag | Description |
|------|-------------|
| `--n N` | Puzzle size (1–9). Default: 4. |
| `--trials N` | Number of puzzles to generate. Default: 100. |
| `--seed N` | Master RNG seed. Default: 0. |
| `--op-policy POLICY` | Operator selection policy. Default: `default`. |
| `--max-solutions N` | Per-puzzle solution count cap. Default: 100. |

```bash
kenken-explorer histogram --n 4 --trials 200 --seed 1
```

Output (formatted for readability; higher solution buckets omitted):

```json
{
  "experiment": "histogram",
  "config": {
    "n": 4,
    "trials": 200,
    "seed": 1,
    "op_policy": "default",
    "size_distribution": {"type": "uniform", "min": 1, "max": 4},
    "max_solutions": 100
  },
  "meta": {"threads": 12, "elapsed_ms": 28},
  "result": {
    "histogram": [
      {"solutions": 1, "count": 82},
      {"solutions": 2, "count": 46},
      {"solutions": 3, "count": 28},
      {"solutions": 4, "count": 12}
    ],
    "cap_bucket": 100,
    "no_solution_count": 0,
    "unique_count": 82,
    "multi_solution_count": 118,
    "unique_rate": 0.41
  }
}
```

`cap_bucket` equals `max_solutions`. Any puzzle with that many or more solutions is recorded as exactly `max_solutions` and lands in this bucket; lower-keyed buckets are exact counts.

A nonzero `no_solution_count` indicates a generator bug — every generated puzzle should have at least one solution by construction.

## Configuration files

Every CLI flag can alternatively be set in a TOML file and passed with `--config`. CLI flags override file values when both are present.

```toml
# histogram.toml
[histogram]
n = 4
trials = 10000
seed = 42
max_solutions = 50

[histogram.size_distribution]
type = "uniform"
min = 1
max = 4
```

```bash
kenken-explorer --config histogram.toml histogram --trials 500
```

The `--trials 500` CLI flag overrides `trials = 10000` from the file; all other values come from the file.

## Reproducibility

Results are fully deterministic given the same `--seed` and `--n`, regardless of thread count. Each trial uses an independent `ChaCha8Rng` seeded from `master_seed + trial_index`; because each trial's seed is fixed by its index rather than its execution order, rayon's parallel scheduling has no effect on the histogram. The only non-deterministic fields in the output are `meta.elapsed_ms` and `meta.threads`.

## Downstream analysis

Because each run prints exactly one JSON object, multiple runs form a valid JSON Lines stream:

```bash
for seed in 1 2 3 4 5; do
  kenken-explorer histogram --n 4 --trials 1000 --seed $seed
done | jq '.result.unique_rate'
```
