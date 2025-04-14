use anyhow::Error;

mod types;

pub use types::*;

use crate::CLIENT;

pub async fn get_projects() -> Result<Vec<TrunkProject>, Error> {
    let response = CLIENT
        .get("https://registry.pgtrunk.io/api/v1/trunk-projects")
        .send()
        .await?;
    let status = response.status();

    if status.is_success() {
        Ok(response.json().await?)
    } else {
        let msg = response.text().await?;

        Err(Error::msg(format!(
            "Failed to fetch trunk projects: {status} {msg}",
        )))
    }
}
