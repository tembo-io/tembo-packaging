use archive::{UnpackedFile, unpack_files};
use log::{debug, trace};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, Write},
    ops::Not,
    path::Path,
    process::ExitCode,
    time::Duration,
};
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

mod archive;
mod digest;

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

    let res = agent
        .get(url.as_str())
        .call()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

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

    debug!("Decompressing in memory");
    let body = res.into_body();
    let (files, digests) = unpack_files(body)?;

    // Validate digests.
    check_digests(&files, &digests)?;

    let tembox_cfg = {
        let tembox_cfg = files
            .iter()
            .find(|file| file.path.as_ref() == Path::new(TEMBOX_CFG))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "tembox.cfg not found"))?;

        std::str::from_utf8(&tembox_cfg.contents).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {err}"))
        })?
    };

    // Validate tembox.cfg.
    check_config(tembox_cfg, name, os)?;

    // Install libs and tembox.cfg.
    copy_libs(&files, LIB_DEST)?;
    // copy_config(name, &dir, TEMBOX_DEST)?;

    Ok(())
}

fn copy_libs(files: &[UnpackedFile], dest: impl AsRef<Path>) -> Result<(), io::Error> {
    println!("   Copying shared libraries...");

    for entry in files {
        let entry_path = entry.path.as_ref();

        if entry_path.starts_with("lib").not() || entry_path.ends_with(".so").not() {
            debug!("skipping {:?}", entry_path);
            continue;
        }
        let Some(file_name) = entry_path.file_name() else {
            debug!("File without file name: {}", entry_path.display());
            continue;
        };

        let dest_file = dest.as_ref().join(file_name);

        println!(
            "     lib/{} -> {}",
            entry_path.file_name().unwrap().to_str().unwrap(),
            dest_file.as_os_str().to_str().unwrap(),
        );

        if entry.entry_type.is_symlink() {
            if let Err(e) = std::fs::remove_file(&dest_file) {
                if e.kind() != io::ErrorKind::NotFound {
                    return Err(e);
                }
            }

            std::os::unix::fs::symlink(entry_path, &dest_file)?;
        } else {
            fs::write(&dest_file, &entry.contents)?;
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

fn check_digests(
    files: &[UnpackedFile],
    digests: &HashMap<Box<Path>, [u8; 64]>,
) -> Result<(), io::Error> {
    print!("   Validating digests...");

    for file in files {
        let expected_digest = digests.get(&file.path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("No digest found for {}", file.path.display()),
            )
        })?;

        let obtained_digest = sha512(&file.contents);
        if &obtained_digest != expected_digest {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Digest mismatch for {}", file.path.display()),
            ));
        }
    }

    println!("OK");
    Ok(())
}

fn sha512(contents: &[u8]) -> [u8; 64] {
    let mut hasher = hmac_sha512::Hash::new();
    hasher.update(contents);
    hasher.finalize()
}

fn check_config(tembox_cfg: &str, pkg: &str, os: &str) -> Result<(), io::Error> {
    print!("   Validating {TEMBOX_CFG}...");

    let cfg = parse_config(tembox_cfg).inspect_err(|_| println!("NOT OK"))?;
    let default = "";

    for (key, want) in [
        ("tembox_package", pkg),
        ("tembox_os", os),
        ("tembox_arch", ARCH),
    ] {
        let got = cfg.get(key).unwrap_or(&default);
        if *got != want {
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
    let lsb_contents = fs::read_to_string(LSB_FILE)?;
    let lsb = parse_config(&lsb_contents)?;
    const OS_KEY: &str = "DISTRIB_CODENAME";
    match lsb.get(OS_KEY) {
        Some(v) => Ok(v.to_string()),
        None => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{OS_KEY} not found in {}", LSB_FILE),
        )),
    }
}

fn parse_config(content: &str) -> Result<HashMap<&str, &str>, io::Error> {
    let mut res = HashMap::new();
    for line in content.lines() {
        trace!("line {line}");
        let mut split = line.splitn(2, "=");
        if let Some(key) = split.next() {
            if let Some(val) = split.last() {
                res.insert(key, val);
            }
        }
    }

    Ok(res)
}
