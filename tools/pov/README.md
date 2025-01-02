# PoV Benchmarking

This script runs a parameterized benchmark with the given `params` and collects the result over them
into a json file. Additionally this file may be used to visualize the following metrics as charts:

- PoV Size
- DB Read/Write counts
- Extrinsic time

This serves as tool for answering the questions in the following category:

- Does the implementation lead to a change in the following metrics for a given input:
  - PoV Size
  - DB Reads/Writes
  - Extrinsic Time
- Why does the PoV size increase/decrease?
- How does an alternate implementation compare on the same metrics?

_Note:_ The numbers obtained via the script have a direct correlation with the specific benchmark that was crafted for this purpose and the provided input.

## Usage

1. Write a benchmark with parameters (any number of them) that establish a specific scenario.

```rust
// pallet_foobar

my_bench {
  let x in 1 .. 100;
  let y in 1 .. 10;

  for _ in 0..x {
    for _ in 0..y {
        // do something
    }
  }
}: my_bench_extrinsic(RawOrigin::Signed(caller))
```

2. Invoke the script

```
ts-node tools/pov/index.ts run \
  --pallet "pallet_foobar" \
  --benchmark "my_bench" \
  --params '10,1 50,5 100,10' \
  --view
```

The above command overrides the parameters `x` and `y` and creates three benchmarked scenarios with explicitly set values: `10,1`, `50,5` and `100,10`.

The original compiled ranges (e.g. `x in 1 .. 100`) are completely ignored, and substituted with the exact command-line parameters defined via `--params` (e.g. `x = 10 50 100`).

3. The execution above will create a `output.json` containing information for the above scenarios and would also open a chart-wise comparison in the browser.

4. Analyze multiple results (optional)

```
ts-node tools/pov/index.ts analyze --input output-1.json output-2.json output-3.json
```

The above command will chart out all the input data on the same chart for better comparisons.
