use flate2::read::GzDecoder;
use log::{debug, trace};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, Write},
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
const LIB_DEST: &str = "/var/lib/postgresql/data/lib";
const TEMBOX_DEST: &str = "/var/lib/postgresql/data/tembox";
const TEMBOX_CFG: &str = "tembox.cfg";
const LSB_FILE: &str = "/etc/lsb-release";
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn main() -> Result<ExitCode, io::Error> {
    let os = get_codename()?;
    debug!("Creating {LIB_DEST}");
    fs::create_dir_all(LIB_DEST)?;
    debug!("Creating {TEMBOX_DEST}");
    fs::create_dir_all(TEMBOX_DEST)?;

    let mut code = ExitCode::SUCCESS;
    for pkg in env::args().skip(1) {
        println!("📦 Installing {pkg}");
        // XXX Make build async and wait for them all to finish.
        match build(&pkg, &os) {
            Ok(_) => println!("✅ {pkg} installed"),
            Err(e) => {
                eprintln!("🚨 {pkg} Error: {e}");
                code = ExitCode::FAILURE;
            }
        }
    }

    Ok(code)
}

fn build(name: &str, os: &str) -> Result<(), io::Error> {
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

    // Decompress tarball into a temporary directory.
    let tmp = tempdir().unwrap();
    debug!("Decompressing into {:?}", tmp);
    let body: &mut ureq::Body = res.body_mut();
    let tar = GzDecoder::new(body.as_reader());
    let mut archive = Archive::new(tar);
    archive.unpack(&tmp)?;

    // Validate digests.
    let dir = tmp.as_ref().join(&pkg);
    check_digests(&dir)?;

    // Validate tembox.cfg.
    check_config(&dir, name, os)?;

    // Install libs and tembox.cfg.
    copy_libs(&dir, LIB_DEST)?;
    copy_config(name, &dir, TEMBOX_DEST)?;

    Ok(())
}

fn copy_libs(dir: &Path, dest: impl AsRef<Path>) -> Result<(), io::Error> {
    println!("   Copying shared libraries...");
    let src = dir.join("lib");

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        if path.ends_with(".so") {
            debug!("skipping {:?}", path);
            continue;
        }
        let dest_file = dest.as_ref().join(entry.file_name());
        let meta = fs::symlink_metadata(&path).unwrap();
        println!(
            "     lib/{} -> {}",
            path.file_name().unwrap().to_str().unwrap(),
            dest_file.as_os_str().to_str().unwrap(),
        );
        if meta.is_symlink() {
            // Just recreate the symlink.
            if let Err(e) = std::fs::remove_file(&dest_file) {
                if e.kind() != io::ErrorKind::NotFound {
                    return Err(e);
                }
            }
            std::os::unix::fs::symlink(fs::read_link(path).unwrap(), &dest_file)?
        } else {
            fs::copy(path, &dest_file)?;
        }
    }
    Ok(())
}

fn copy_config(pkg: &str, dir: &Path, dest: impl AsRef<Path>) -> Result<(), io::Error> {
    let src = dir.join(TEMBOX_CFG);
    let tembox = dest.as_ref().join(format!("{pkg}.cfg"));
    println!(
        "     {TEMBOX_CFG} -> {}",
        tembox.as_os_str().to_str().unwrap(),
    );
    fs::copy(&src, &tembox)?;

    Ok(())
}

fn check_digests(dir: &Path) -> Result<(), io::Error> {
    print!("   Validating digests...");
    io::stdout().flush()?;
    Command::new("sha512sum")
        .args(["--check", "--strict", "--quiet", "digests"])
        .current_dir(dir)
        .spawn()
        .expect("digests validation failed")
        .wait()
        .inspect_err(|_| println!("NOT OK"))?;
    println!("OK");
    Ok(())
}

fn check_config(dir: &Path, pkg: &str, os: &str) -> Result<(), io::Error> {
    print!("   Validating {TEMBOX_CFG}...");
    io::stdout().flush()?;
    let cfg = parse_config(dir.join(TEMBOX_CFG)).inspect_err(|_| println!("NOT OK"))?;
    let default = "".to_string();

    for (key, want) in [
        ("tembox_package", pkg),
        ("tembox_os", os),
        ("tembox_arch", ARCH),
    ] {
        let got = cfg.get(key).unwrap_or(&default);
        if got != want {
            println!("NOT OK");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Wrong {key}: Expected {:?} but got {:?}", want, got),
            ));
        }
    }

    println!("OK");
    Ok(())
}

fn get_codename() -> Result<String, io::Error> {
    let lsb = parse_config(LSB_FILE)?;
    const OS_KEY: &str = "DISTRIB_CODENAME";
    match lsb.get(OS_KEY) {
        Some(v) => Ok(v.to_string()),
        None => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{OS_KEY} not found in {}", LSB_FILE),
        )),
    }
}

fn parse_config(file: impl AsRef<Path>) -> Result<HashMap<String, String>, io::Error> {
    use std::io::BufRead;
    debug!("Parsing {:?}", file.as_ref());
    let file = File::open(file)?;
    let reader = io::BufReader::new(file);
    let mut res = HashMap::new();
    for line in reader.lines().map_while(Result::ok) {
        trace!("line {line}");
        let mut split = line.splitn(2, "=");
        if let Some(key) = split.next() {
            if let Some(val) = split.last() {
                res.insert(key.to_string(), val.to_string());
            }
        }
    }
    Ok(res)
}
