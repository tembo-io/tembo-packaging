use flate2::read::GzDecoder;
use log::{debug, trace};
use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
    process::{Command, ExitCode},
    time::Duration,
};
use tar::Archive;
use tempfile::tempdir;
use ureq::{Agent, http::StatusCode};

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "amd64";

#[cfg(target_arch = "aarch64")]
const ARCH: &str = "arm64";
const URL: &str = "https://cdb-plat-use1-prod-pgtrunkio.s3.us-east-1.amazonaws.com/dependencies";
static DEST: &str = "/var/lib/postgresql/data/lib";
const LSB_FILE: &str = "/etc/lsb-release";
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() -> Result<ExitCode, io::Error> {
    let os = get_codename()?;
    let dest = format!("{DEST}/{os}");
    debug!("Creating {dest}");
    fs::create_dir_all(&dest)?;

    let mut code = ExitCode::SUCCESS;
    for pkg in env::args().skip(1) {
        println!("📦 Installing {pkg}");
        // XXX Make build async and wait for them all to finish.
        match build(&pkg, &dest, &os) {
            Ok(_) => println!("✅ {pkg} installed"),
            Err(e) => {
                eprintln!("🚨 {pkg} Error: {e}");
                code = ExitCode::FAILURE;
            }
        }
    }

    Ok(code)
}

fn build(name: &str, dest: impl AsRef<Path>, os: &str) -> Result<(), io::Error> {
    println!("   Downloading {name}");
    let pkg = format!("tembo-{name}_{ARCH}");
    let url = format!("{URL}/{os}/{pkg}.tgz");
    debug!("Downloading {url}");

    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .https_only(true)
        .user_agent(APP_USER_AGENT)
        .build()
        .into();

    let mut res = match agent.get(url.as_str()).call() {
        Ok(r) => r,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("{e}"))),
    };

    match res.status() {
        StatusCode::OK => (),
        StatusCode::NOT_FOUND => {
            return Err(io::Error::new(io::ErrorKind::NotFound, "not found"));
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("download failed: {}", res.status()),
            ));
        }
    }

    // Decompress gzipped data into a tar archive.
    let tmp = tempdir().unwrap();
    debug!("Decompressing into {:?}", tmp);
    let body: &mut ureq::Body = res.body_mut();
    let tar = GzDecoder::new(body.as_reader());
    let mut archive = Archive::new(tar);
    archive.unpack(&tmp)?;

    // Validate digests.
    print!("   Validating digests...");
    Command::new("sha512sum")
        .args(["--check", "--strict", "--quiet", "digests"])
        .current_dir(tmp.as_ref().join(&pkg))
        .spawn()
        .expect("digests validation failed")
        .wait()
        .unwrap();
    println!("OK");

    // Install libs.
    println!("   Copying shared libraries...");
    for entry in fs::read_dir(tmp.as_ref().join(&pkg).join("lib"))? {
        let entry = entry?;
        let path = entry.path();
        if path.ends_with(".so") {
            debug!("skipping {:?}", path);
            continue;
        }
        let dest = dest.as_ref().join(entry.file_name());
        let meta = fs::symlink_metadata(&path).unwrap();
        println!(
            "     lib/{} -> {}",
            path.file_name().unwrap().to_str().unwrap(),
            dest.as_os_str().to_str().unwrap(),
        );
        if meta.is_symlink() {
            // Just recreate the symlink.
            if let Err(e) = std::fs::remove_file(&dest) {
                if e.kind() != io::ErrorKind::NotFound {
                    return Err(e);
                }
            }
            std::os::unix::fs::symlink(fs::read_link(path).unwrap(), &dest)?
        } else {
            fs::copy(path, &dest)?;
        }
    }
    Ok(())
}

fn get_codename() -> Result<String, io::Error> {
    use std::io::BufRead;
    debug!("Parsing {LSB_FILE}");
    let file = File::open(LSB_FILE)?;
    let reader = io::BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        trace!("line {line}");
        let mut split = line.splitn(2, "=");
        if let Some(key) = split.next() {
            if key == "DISTRIB_CODENAME" {
                if let Some(val) = split.last() {
                    return Ok(val.to_string());
                }
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("DISTRIB_CODENAME not found in {}", LSB_FILE),
    ))
}
