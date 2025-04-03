use std::{thread::sleep, time::Duration};

use benchmark_macro::bench;

#[bench]
fn main() {
    env_logger::init();

    println!("Hello, world!");
    #[bench("foo")]
    {
        sleep(Duration::from_secs(5));
    }

    if true {
        #[bench("bar")]
        sleep(Duration::from_secs(5));
    }
}
