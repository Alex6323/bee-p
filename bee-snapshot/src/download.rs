// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{config::SnapshotConfig, Error};

use log::{error, info, warn};

use std::{fs::File, io::copy, path::Path};

// TODO copy is not really streaming ?
// TODO temporary file until fully downloaded ?
pub async fn download_snapshot(config: &SnapshotConfig) -> Result<(), Error> {
    let path = config.path();

    let config = config.clone();

    for url in config.download_urls() {
        info!("Downloading snapshot file from {}...", url);
        match reqwest::get(url).await {
            Ok(res) => match File::create(config.path()) {
                // TODO unwrap
                Ok(mut file) => match copy(&mut res.bytes().await.unwrap().as_ref(), &mut file) {
                    Ok(_) => break,
                    Err(e) => warn!("Copying snapshot file failed: {:?}.", e),
                },
                Err(e) => warn!("Creating snapshot file failed: {:?}.", e),
            },
            Err(e) => warn!("Downloading snapshot file failed: {:?}.", e),
        }
    }

    // TODO here or outside ?
    if Path::new(path).exists() {
        Ok(())
    } else {
        error!("No working download source available.");
        Err(Error::NoDownloadSourceAvailable)
    }
}
