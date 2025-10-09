// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Context, Result, anyhow, bail};
use std::{fs, path::PathBuf};
use tracing::{debug, error, instrument};

/// Check if a directory exists, if not create it including all
/// parent components
#[instrument]
pub fn check_dir_or_create(path: &PathBuf) -> Result<()> {
    if !path.is_dir() {
        debug!("directory `{}` does not exist, creating it", path.display());
        return fs::create_dir_all(path)
            .inspect_err(|e| error!("Failed to create directory `{}`: {e}", path.display()))
            .context(format!("Failed to create directory `{}`", path.display()));
    }
    Ok(())
}

/// Check if a directory exists
#[instrument]
pub fn check_dir(path: &str) -> Result<()> {
    let meta = fs::metadata(path).map_err(|e| {
        error!("failed to read {}: {}", path, e);
        anyhow!("failed to read {}: {}", path, e)
    })?;
    if !meta.is_dir() {
        error!("{path}: not a directory");
        bail!("{path}: not a directory");
    }
    Ok(())
}
