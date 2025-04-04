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
    sleep(Duration::from_secs(1));

    for _ in 0..5 {
        #[bench("baz")]
        sleep(Duration::from_secs(1));
    }
}
```

```sh
❯ RUST_LOG="debug" cargo run
   Compiling benchmark-macro v0.1.0 (/home/enoki/Projects/benchmark-macro)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.84s
     Running `target/debug/benchmark-macro`
[2025-04-04T15:30:11Z DEBUG benchmark_macro] Benchmark for foo: 1000105 microseconds over 1 run(s)
[2025-04-04T15:30:11Z DEBUG benchmark_macro] Benchmark for baz: 5000483 microseconds over 5 run(s)
```
