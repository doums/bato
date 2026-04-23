// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Context, Result};
use bato::{Bato, Config, cli::Cli, signal, trace};
use clap::Parser;
use std::time::Duration;
use tracing::{debug, instrument, trace};

#[instrument]
fn main() -> Result<()> {
    let cli = Cli::parse();
    let _g = trace::init(&cli).context("failed to init tracing")?;
    debug!("{:?}", cli);

    signal::catch_signals()?;

    let config = Config::new(cli.config)?;
    trace!("{:#?}", config);
    debug!("tick rate {}s", config.tick_rate);
    let tick = Duration::from_secs(config.tick_rate as u64);
    let mut bato = Bato::with_config(config)?;
    debug!("{:#?}", bato);

    bato.run(tick)?;

    Ok(())
}
