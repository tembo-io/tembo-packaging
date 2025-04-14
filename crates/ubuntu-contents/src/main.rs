use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter},
    thread,
};

use anyhow::bail;
use flate2::read::GzDecoder;
use regex::bytes::Regex;

fn generate_mapping(version: &str) -> anyhow::Result<HashMap<String, String>> {
    let mut map = HashMap::new();

    let pattern = Regex::new(
        r#".*\/(?<library>[^\/\s]+\.so(?:\.\d+(?:\.\d+)*)?)(?<whitespace>\s+)(?<package>.*)"#,
    )
    .unwrap();
    let url = format!("http://archive.ubuntu.com/ubuntu/dists/{version}-updates/Contents-amd64.gz");
    println!(
        "[{:?}] Will download and unpack the contents of {url}. This could take a while.",
        thread::current().id()
    );

    let body = ureq::get(url).call()?.into_body();

    let decoder = GzDecoder::new(body.into_reader());
    let mut reader = BufReader::new(decoder);

    let mut buf = String::new();

    loop {
        let bytes_read = reader.read_line(&mut buf)?;

        if bytes_read == 0 {
            break;
        }

        if let Some(captures) = pattern.captures(buf.as_bytes()) {
            let library = captures
                .name("library")
                .and_then(|capture| str::from_utf8(capture.as_bytes()).ok())
                .expect("No `library`, but it was expected");

            let package = captures
                .name("package")
                .and_then(|capture| str::from_utf8(capture.as_bytes()).ok())
                .expect("No `package`, but it was expected");

            // Stores the full path of the project, e.g. `universe/libs/package` (for something in the community-maintained repo) or `libs/project` (for something in the main repo)
            //
            // If multiple packages supply the same library, the one with the shortest name is chosen.
            // This means that packages in the main repo are preferred over those in the community-maintained repo,
            // and packages with shorter names are preferred over those with longer names.
            // This means that libc6 is preferred over libc6-x32, which itself is preferred over libc6-i386
            map.entry(library.to_string())
                .and_modify(|current_entry: &mut String| {
                    if package.len() < current_entry.len() {
                        *current_entry = package.to_string();
                    }
                })
                .or_insert_with(|| package.to_string());
        }

        buf.clear();
    }

    Ok(map
        .into_iter()
        .map(|(library_name, package_name)| {
            // Now that all packages are collected and sorted,
            // let's strip away the repository info
            //
            // `universe/libs/project` becomes just `project`
            let package_name = package_name.split('/').last().unwrap_or(&package_name);

            (library_name, package_name.to_string())
        })
        .collect())
}

fn generate_library_mapping_json(ubuntu_version: &str) -> anyhow::Result<()> {
    let mapping = generate_mapping(ubuntu_version)?;

    let output_path = format!("./library_mapping_{}.json", ubuntu_version);

    println!("[{:?}] Writing to {output_path}", thread::current().id());

    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &mapping)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let versions = ["focal", "jammy"];
    let mut handles = Vec::with_capacity(versions.len());
    println!(
        "[{:?}] Spawning {} threads",
        thread::current().id(),
        versions.len()
    );

    for version in versions {
        let handle = thread::spawn(|| generate_library_mapping_json(version));

        handles.push(handle);
    }

    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => (),
            Ok(Err(err)) => bail!("Task failed: {err}"),
            Err(err) => bail!("Failed to join thread: {err:?}"),
        }
    }

    Ok(())
}
