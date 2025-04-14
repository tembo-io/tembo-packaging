use std::{collections::HashMap, io, ops::Not, sync::LazyLock};

use anyhow::Context;
use dependencies::Dependencies;
use slicedisplay::SliceDisplay;
use tokio::task::{JoinSet, spawn_blocking};
use trunk::TrunkProject;

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

mod dependencies;
/// Utilities for interfacing with the Trunk API
mod trunk;
/// Unpack a tar.gz in-memory
mod unpack;

async fn identify_dependencies(project: TrunkProject) -> anyhow::Result<(String, Dependencies)> {
    let download =
        project.downloads.into_iter().next().with_context(|| {
            format!("Found no download link for Trunk project {}", project.name)
        })?;

    let contents = CLIENT.get(download.link).send().await?.bytes().await?;

    let task = spawn_blocking(move || {
        let archive = unpack::decompress_in_memory(&contents)?;
        drop(contents);

        let dependencies = Dependencies::read_from_archive(archive)?;

        Ok(dependencies) as anyhow::Result<_>
    });

    let dependencies = task.await??;

    Ok((project.name, dependencies))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut join_set = JoinSet::new();
    let projects = trunk::get_projects().await?;
    let mut dependency_map = HashMap::new();

    for project in projects {
        join_set.spawn(async move { identify_dependencies(project).await });
    }

    while let Some(join_result) = join_set.join_next().await {
        let task_result = match join_result {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Failed to join task: {err}");
                continue;
            }
        };

        match task_result {
            Ok((proj_name, deps)) => {
                dependency_map.insert(proj_name, deps);
            }
            Err(err) => eprintln!("Task failed: {err}"),
        }
    }

    let mut stdout = io::stdout();
    write_dependencies_yaml(&mut stdout, dependency_map)?;

    Ok(())
}

fn write_dependencies_yaml(
    writer: &mut dyn std::io::Write,
    dependency_map: HashMap<String, Dependencies>,
) -> anyhow::Result<()> {
    for (project_name, dependency) in dependency_map {
        let suppliers: Vec<_> = dependency
            .suppliers
            .into_values()
            .filter(|supplier| supplier.is_libc().not())
            .map(|supplier| supplier.to_string())
            .collect();
        if suppliers.is_empty() {
            continue;
        }

        writeln!(writer, "{project_name}: {}", suppliers.display())?;
    }

    Ok(())
}
