// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod error;
use bato::{deserialize_config, Bato, Config};
use std::io::Error;
use std::process;
use std::thread;
use std::time::Duration;

const TICK_RATE: Duration = Duration::from_secs(5);

fn main() -> Result<(), Error> {
    let config = deserialize_config().unwrap_or_else(|err| {
        eprintln!("bato error: {}", err);
        process::exit(1);
    });
    let mut bato = Bato::with_config(config);
    loop {
        bato.check().unwrap_or_else(|err| {
            eprintln!("bato error: {}", err);
            process::exit(1);
        });
        thread::sleep(TICK_RATE);
    }
    Ok(())
}
