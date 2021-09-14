// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod error;
use bato::{Bato, Config};
use std::io::Error;
use std::process;
use std::thread;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_secs(5);

fn main() -> Result<(), Error> {
    let mut config = Config::new().unwrap_or_else(|err| {
        eprintln!("bato error: {}", err);
        process::exit(1);
    });
    config.normalize();
    let mut tick = TICK_RATE;
    if let Some(rate) = config.tick_rate {
        tick = Duration::from_secs(rate as u64);
    }
    let mut bato = Bato::with_config(&config).unwrap_or_else(|err| {
        eprintln!("bato error: {}", err);
        process::exit(1);
    });
    bato.start().unwrap_or_else(|err| {
        eprintln!("bato error: {}", err);
        process::exit(1);
    });
    loop {
        bato.update(&config).unwrap_or_else(|err| {
            eprintln!("bato error: {}", err);
            process::exit(1);
        });
        thread::sleep(tick);
    }
    bato.close();
    Ok(())
}
