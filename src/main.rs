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
