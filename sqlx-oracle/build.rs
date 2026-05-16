use std::path::Path;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    enum PkgKind { Zip, Dmg }

    // Verified download URLs from https://www.oracle.com/database/technologies/instant-client/
    // macOS ARM64 / Intel: permanent links (serve latest version)
    // Linux: permanent link
    // Windows: versioned URL (no permanent link available for otn_software)
    let win_ver = "23.26.1.0.0";
    let win_code = "2326100";

    let pkg = match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => (
            "https://download.oracle.com/otn_software/mac/instantclient/instantclient-basiclite-macos-arm64.dmg".to_owned(),
            PkgKind::Dmg,
        ),
        ("macos", "x86_64") => (
            "https://download.oracle.com/otn_software/mac/instantclient/instantclient-basiclite-macos.dmg".to_owned(),
            PkgKind::Dmg,
        ),
        ("linux", "x86_64") => (
            "https://download.oracle.com/otn_software/linux/instantclient/instantclient-basiclite-linuxx64.zip".to_owned(),
            PkgKind::Zip,
        ),
        ("linux", "aarch64") => (
            "https://download.oracle.com/otn_software/linux/instantclient/2326100/instantclient-basic-linux.arm64-23.26.1.0.0.zip".to_owned(),
            PkgKind::Zip,
        ),
        ("windows", "x86_64") => (
            format!("https://download.oracle.com/otn_software/nt/instantclient/{win_code}/instantclient-basiclite-windows.x64-{win_ver}.zip"),
            PkgKind::Zip,
        ),

        _ => panic!("sqlx-oracle: unsupported target {target_os}/{target_arch}"),
    };

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_dir = Path::new(&out_dir).join("instantclient");

    // Marker: we assume extraction is done if lib_dir exists and contains libclntsh
    let marker = lib_dir.join(".extracted");

    if !marker.exists() {
        println!("cargo:warning=Downloading Oracle Instant Client Basic Light for {target_os}/{target_arch} ...");

        if lib_dir.exists() {
            std::fs::remove_dir_all(&lib_dir).unwrap();
        }
        std::fs::create_dir_all(&lib_dir).unwrap();

        match pkg.1 {
            PkgKind::Zip => extract_zip(&pkg.0, &lib_dir),
            PkgKind::Dmg => extract_dmg(&pkg.0, &lib_dir),
        }

        // After extraction, ensure libclntsh.dylib / libclntsh.so exists (create symlink if needed)
        ensure_libclntsh(&lib_dir, target_os.as_str());

        std::fs::write(&marker, b"").unwrap();
        println!("cargo:warning=Oracle Instant Client ready at {}", lib_dir.display());
    }

    println!("cargo:rustc-link-search={}", lib_dir.display());

    if target_os == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_ORACLE");
}

fn run(prog: &str, args: &[&str]) {
    let status = std::process::Command::new(prog)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to run `{prog}`: {e}"));
    assert!(status.success(), "`{prog}` failed");
}

fn run_output(prog: &str, args: &[&str]) -> String {
    let out = std::process::Command::new(prog)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run `{prog}`: {e}"));
    assert!(out.status.success(), "`{prog}` failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn download(url: &str, dest: &Path) {
    run("curl", &["-L", "-o", &dest.to_string_lossy(), "--fail", url]);
}

fn extract_zip(url: &str, dest: &Path) {
    let archive = dest.join("pkg.zip");
    download(url, &archive);
    // unzip to parent so files land in dest (we cd'd via -d dest)
    run("unzip", &["-q", "-o", &archive.to_string_lossy(), "-d", &dest.to_string_lossy()]);
    let _ = std::fs::remove_file(&archive);
    // Flatten: if a single subdir was created, move its contents up
    flatten_single_subdir(dest);
}

fn extract_dmg(url: &str, dest: &Path) {
    let dmg = dest.join("pkg.dmg");
    download(url, &dmg);

    // Mount and capture mount point
    let out = run_output("hdiutil", &[
        "attach", "-nobrowse", "-mountrandom", "/tmp",
        &dmg.to_string_lossy(),
    ]);
    let mount_point = out.lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            parts.last().filter(|p| p.starts_with('/')).map(|s| s.to_string())
        })
        .next()
        .expect("failed to find mount point in hdiutil output");

    run("cp", &["-Rp", &format!("{mount_point}/."), &dest.to_string_lossy()]);
    run("hdiutil", &["detach", &mount_point]);
    let _ = std::fs::remove_file(&dmg);
}

/// If the extraction created a single subdirectory, move everything up one level.
fn flatten_single_subdir(dir: &Path) {
    let entries: Vec<_> = std::fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    if entries.len() == 1 {
        let sub = entries[0].path();
        for entry in std::fs::read_dir(&sub).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            let dest_path = dir.join(&name);
            if dest_path.exists() {
                let _ = std::fs::remove_dir_all(&dest_path);
            }
            std::fs::rename(&entry.path(), &dest_path).unwrap_or_else(|_| {
                // fallback: copy & remove
                run("cp", &["-Rp", &entry.path().to_string_lossy(), &dest_path.to_string_lossy()]);
                let _ = std::fs::remove_dir_all(&entry.path());
            });
        }
        let _ = std::fs::remove_dir_all(&sub);
    }
}

/// Find libclntsh* and create `libclntsh.dylib` (or .so) symlink if it doesn't exist.
fn ensure_libclntsh(dir: &Path, os: &str) {
    let want = format!("libclntsh.{}", match os {
        "macos" => "dylib",
        "linux" => "so",
        "windows" => "dll",
        _ => unreachable!(),
    });
    let target = dir.join(&want);

    // Already present?
    if target.exists() {
        return;
    }

    // Look for any libclntsh* file
    let found: Vec<_> = std::fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .filter(|n| n.to_string_lossy().starts_with("libclntsh"))
        .collect();

    if found.is_empty() {
        panic!(
            "libclntsh not found in {}.\n  Contents: {:?}",
            dir.display(),
            std::fs::read_dir(dir).map(|e| e.filter_map(|e| e.ok()).map(|e| e.file_name()).collect::<Vec<_>>()).unwrap_or_default()
        );
    }

    // Create symlink from the first found file
    let src = found[0].clone();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&src, &target).unwrap();
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(&src, &target).unwrap();
    }
}
