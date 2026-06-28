//! 构建脚本 —— 自动下载并链接 Oracle Instant Client。
//!
//! sibyl crate 的 build.rs 固定输出 `cargo:rustc-link-lib=dylib=clntsh`，
//! 因此必须在链接前确保 `libclntsh.dylib` / `.so` / `.dll` 可用。
//! 本脚本根据目标平台自动下载对应 Oracle Instant Client Basic Light 包，
//! 解压到 `OUT_DIR/instantclient`，并设置 `rustc-link-search` 和 rpath。

use std::path::Path;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    enum PkgKind { Zip, Dmg }

    // Oracle Instant Client 下载地址（来自 Oracle 官方永久链接）
    // macOS: 永久链接，自动指向最新版本
    // Linux / Windows: 固定版本号
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

    // 标记文件：存在即表示已解压完成
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

        // 确保 `libclntsh.dylib` / `libclntsh.so` 存在（必要时创建符号链接）
        ensure_libclntsh(&lib_dir, target_os.as_str());

        std::fs::write(&marker, b"").unwrap();
        println!("cargo:warning=Oracle Instant Client ready at {}", lib_dir.display());
    }

    // 通知链接器搜索路径
    println!("cargo:rustc-link-search={}", lib_dir.display());

    // Linux: 允许 libclntsh.so 中未解析的符号（Oracle 内部符号通过 dlopen 在运行时加载）
    if target_os == "linux" {
        println!("cargo:rustc-link-arg=-Wl,--allow-shlib-undefined");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }

    // macOS 处理 dylib 的 install name 和代码签名
    if target_os == "macos" {
        let libclntsh = lib_dir.join("libclntsh.dylib.23.1");
        if libclntsh.exists() {
            let lp = libclntsh.to_string_lossy();
            let nnz = lib_dir.join("libnnz.dylib");
            let nnz_s = nnz.to_string_lossy();
            let core = lib_dir.join("libclntshcore.dylib.23.1");
            let core_s = core.to_string_lossy();

            // 将 libclntsh 自身 install name 改为绝对路径，下游二进制可直接加载
            run("install_name_tool", &["-id", &lp, &lp]);
            // libclntsh 内部引用 @rpath/libnnz.dylib 和 @rpath/libclntshcore.dylib.23.1，
            // 改为绝对路径以便无需 rpath 即可加载
            run("install_name_tool", &["-change", "@rpath/libnnz.dylib", &nnz_s, &lp]);
            run("install_name_tool", &["-change", "@rpath/libclntshcore.dylib.23.1", &core_s, &lp]);
            if nnz.exists() {
                run("install_name_tool", &["-id", &nnz_s, &nnz_s]);
            }
            if core.exists() {
                run("install_name_tool", &["-id", &core_s, &core_s]);
            }

            // ad-hoc 签名：macOS AMFI 在直接执行时要求所有加载的 dylib 有合法签名
            //（lldb 调试时可绕过此检查，但正常启动会被 SIGKILL 终止）
            run("codesign", &["-f", "-s", "-", &lp]);
            run("codesign", &["-f", "-s", "-", &nnz_s]);
            run("codesign", &["-f", "-s", "-", &core_s]);
        }
        // rpath 仅对本 crate 自身有效（不会传播到下游二进制），作为备选保留
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
    }

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_ORACLE");
}

/// 运行外部命令并检查是否成功。
fn run(prog: &str, args: &[&str]) {
    let status = std::process::Command::new(prog)
        .args(args)
        .status()
        .unwrap_or_else(|e| panic!("failed to run `{prog}`: {e}"));
    assert!(status.success(), "`{prog}` failed");
}

/// 运行外部命令并捕获 stdout。
fn run_output(prog: &str, args: &[&str]) -> String {
    let out = std::process::Command::new(prog)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run `{prog}`: {e}"));
    assert!(out.status.success(), "`{prog}` failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// 使用 curl 下载文件。
fn download(url: &str, dest: &Path) {
    run("curl", &["-L", "-o", &dest.to_string_lossy(), "--fail", url]);
}

/// 解压 .zip 包到目标目录，并展平单子目录结构。
fn extract_zip(url: &str, dest: &Path) {
    let archive = dest.join("pkg.zip");
    download(url, &archive);
    run("unzip", &["-q", "-o", &archive.to_string_lossy(), "-d", &dest.to_string_lossy()]);
    let _ = std::fs::remove_file(&archive);
    // 如果解压后只有一个子目录，将其内容上移一层
    flatten_single_subdir(dest);
}

/// 挂载 .dmg、复制内容并弹出。
fn extract_dmg(url: &str, dest: &Path) {
    let dmg = dest.join("pkg.dmg");
    download(url, &dmg);

    // 挂载 DMG 并捕获挂载点路径
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

/// 如果解压创建了单一子目录或 `instantclient_*` 子目录，将其内容上移并删除空壳。
fn flatten_single_subdir(dir: &Path) {
    let entries: Vec<_> = std::fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    // 查找 instantclient_* 子目录（Oracle zip 内含版本目录如 instantclient_23_26）
    let sub = entries.iter()
        .find(|e| e.file_name().to_string_lossy().starts_with("instantclient"))
        .map(|e| e.path())
        .or_else(|| {
            if entries.len() == 1 { Some(entries[0].path()) } else { None }
        });
    if let Some(sub) = sub {
        for entry in std::fs::read_dir(&sub).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            let dest_path = dir.join(&name);
            if dest_path.exists() {
                let _ = std::fs::remove_dir_all(&dest_path);
            }
            std::fs::rename(&entry.path(), &dest_path).unwrap_or_else(|_| {
                // 跨设备回退：复制后删除
                run("cp", &["-Rp", &entry.path().to_string_lossy(), &dest_path.to_string_lossy()]);
                let _ = std::fs::remove_dir_all(&entry.path());
            });
        }
        let _ = std::fs::remove_dir_all(&sub);
    }
}

/// 在 Instant Client 目录中找到 `libclntsh*` 并确保 `libclntsh.{dylib,so,dll}` 存在。
///
/// Oracle 分发的 dylib 文件名带版本后缀（如 `libclntsh.dylib.23.1`），
/// 链接器需要精确的 `libclntsh.dylib` 名称，此处创建符号链接。
fn ensure_libclntsh(dir: &Path, os: &str) {
    let want = format!("libclntsh.{}", match os {
        "macos" => "dylib",
        "linux" => "so",
        "windows" => "dll",
        _ => unreachable!(),
    });
    let target = dir.join(&want);

    if target.exists() {
        return;
    }

    // 查找任意 libclntsh 开头的文件
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

    // 为第一个找到的版本化文件创建符号链接
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
