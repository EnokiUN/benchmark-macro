# Benchmark Marco

A simple (badly implemented) proc macro to potentially simplify benchmarking
parts of code using `std::time::Instant`s.

## Example

```rs
use std::{thread::sleep, time::Duration};

use benchmark_macro::bench;

#[bench]
fn main() {
    env_logger::init();

    #[bench("foo")]
    {
        sleep(Duration::from_secs(5));
    }

    if true {
        #[bench("bar")]
        sleep(Duration::from_secs(5));
    }
}
```

```sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.73s
❯ RUST_LOG="debug" cargo run
   Compiling benchmark-macro v0.1.0 (/home/enoki/Projects/benchmark-macro)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.50s
     Running `target/debug/benchmark-macro`
[2025-04-03T20:21:43Z DEBUG benchmark_macro] Benchmark for foo: 5000103 microseconds
[2025-04-03T20:21:43Z DEBUG benchmark_macro] Benchmark for bar: 5000092 microseconds
```
