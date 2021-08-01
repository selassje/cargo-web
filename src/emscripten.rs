use std::path::{Path, PathBuf};
use std::process::exit;

extern crate git2;
use package::{download_package, PrebuiltPackage};
use utils::find_cmd;

fn emscripten_package() -> Option<PrebuiltPackage> {
    let package = if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        PrebuiltPackage {
                url: "https://github.com/koute/emscripten-build/releases/download/emscripten-1.38.19-1/emscripten-1.38.19-1-x86_64-unknown-linux-gnu.tgz",
                name: "emscripten",
                version: "1.38.19-1",
                arch: "x86_64-unknown-linux-gnu",
                hash: "baab5f1162901bfa220cb009dc628300c5e67b91cf58656ab6bf392d513bff9c",
                size: 211505607
            }
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86") {
        PrebuiltPackage {
                url: "https://github.com/koute/emscripten-build/releases/download/emscripten-1.38.19-1/emscripten-1.38.19-1-i686-unknown-linux-gnu.tgz",
                name: "emscripten",
                version: "1.38.19-1",
                arch: "i686-unknown-linux-gnu",
                hash: "6d211eb0e9bbf82a1bf0dcc336486aa5191952f3938b7c0cf76b8d6946d4c117",
                size: 223770839
            }
    } else {
        return None;
    };

    Some(package)
}

fn binaryen_package() -> Option<PrebuiltPackage> {
    let package = if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        PrebuiltPackage {
                url: "https://github.com/koute/emscripten-build/releases/download/emscripten-1.38.19-1/binaryen-1.38.19-1-x86_64-unknown-linux-gnu.tgz",
                name: "binaryen",
                version: "1.38.19-1",
                arch: "x86_64-unknown-linux-gnu",
                hash: "af079258c6f13234541d932b873762910951779c4682fc917255716637383dc9",
                size: 15818455
            }
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86") {
        PrebuiltPackage {
                url: "https://github.com/koute/emscripten-build/releases/download/emscripten-1.38.19-1/binaryen-1.38.19-1-i686-unknown-linux-gnu.tgz",
                name: "binaryen",
                version: "1.38.19-1",
                arch: "i686-unknown-linux-gnu",
                hash: "9fd0e30d1760d29e3c96fa24592a35629876316fadb7ef882b9c6d8b2eafb0d8",
                size: 15951181
            }
    } else {
        return None;
    };

    Some(package)
}

fn check_emscripten() {
    let possible_commands = if cfg!(windows) {
        &["emcc.bat"]
    } else {
        &["emcc"]
    };

    if find_cmd(possible_commands).is_some() {
        return;
    }

    eprintln!("error: you don't have Emscripten installed!");
    eprintln!("");

    if Path::new("/usr/bin/pacman").exists() {
        eprintln!("You can most likely install it like this:");
        eprintln!("  sudo pacman -S emscripten");
    } else if Path::new("/usr/bin/apt-get").exists() {
        eprintln!("You can most likely install it like this:");
        eprintln!("  sudo apt-get install emscripten");
    } else if cfg!(target_os = "linux") {
        eprintln!("You can most likely find it in your distro's repositories.");
    } else if cfg!(target_os = "windows") {
        eprintln!( "Download and install emscripten from the official site: http://kripken.github.io/emscripten-site/docs/getting_started/downloads.html" );
    }

    if cfg!(unix) {
        if cfg!(target_os = "linux") {
            eprintln!("If not you can install it manually like this:");
        } else {
            eprintln!("You can install it manually like this:");
        }
        eprintln!( "  curl -O https://s3.amazonaws.com/mozilla-games/emscripten/releases/emsdk-portable.tar.gz" );
        eprintln!("  tar -xzf emsdk-portable.tar.gz");
        eprintln!("  source emsdk-portable/emsdk_env.sh");
        eprintln!("  emsdk update");
        eprintln!("  emsdk install sdk-incoming-64bit");
        eprintln!("  emsdk activate sdk-incoming-64bit");
    }

    exit(101);
}

pub struct EmscriptenPackage {
    pub git: String,
    pub rev: String,
    pub sdk_version: String,
}

pub fn setup_emscripten(emscripten: &EmscriptenPackage, destination: &Path) -> Result<(), String> {
    get_emscripten_revision(emscripten, destination)?;
    install_emscripten(emscripten, destination)?;
    Ok(())
}

pub fn install_emscripten(
    emscripten: &EmscriptenPackage,
    destination: &Path,
) -> Result<(), String> {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    let command = "emsdk.bat";
    #[cfg(target_os = "linux")]
    let command = "./emsdk";

    let command = destination.join(command);

    let mut install = Command::new(command.clone());
    install
        .arg("install")
        .arg(emscripten.sdk_version.clone())
        .current_dir(destination);

    let run = |command: &mut Command, err: &str| {
        let output = command.output().map_err(|e| e.to_string())?;
        println!("{}", String::from_utf8(output.stdout).unwrap());
        if output.status.success() {
            Ok(())
        } else {
            let process_error_msg = String::from_utf8(output.stderr).unwrap();
            Err(err.to_owned() + process_error_msg.as_str())
        }
    };

    println!("Executing Emscripten SDK install command");
    run(&mut install, "Failed to install EMSDK : ")?;

    let mut activate_command = Command::new(command);
    activate_command
        .arg("activate")
        .arg(emscripten.sdk_version.clone())
        .current_dir(destination);

    println!("Executing Emscripten SDK activate command");
    run(
        &mut activate_command,
        "Failed to activate Emscripten SDK : ",
    )?;

    if let Some(path) = std::env::var_os("PATH") {
        let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
        paths.push(PathBuf::from(destination));
        let new_path = std::env::join_paths(paths).map_err(|_| "Could not join paths")?;
        std::env::set_var("PATH", &new_path);
    }
    println!(
        "PATH after EmscriptenInstallation {:?}",
        std::env::var_os("PATH").unwrap(),
    );

    Ok(())
}

pub fn get_emscripten_revision(
    emscripten: &EmscriptenPackage,
    destination: &Path,
) -> Result<(), String> {
    println!("Getting the Emscripten SDK repo");

    let repo = git2::Repository::open(destination)
        .or_else(|_| git2::Repository::clone(emscripten.git.as_str(), destination))
        .map_err(|e| "Could not get the Emscripten SDK repo: ".to_owned() + e.message())?;

    let (object, reference) = repo
        .revparse_ext(&emscripten.rev)
        .map_err(|e| "Could not find the Emscripten SDK revision: ".to_owned() + e.message())?;
    let _ = repo
        .checkout_tree(&object, None)
        .map_err(|e| "Could not checkout the Emscripten commit: ".to_owned() + e.message())?;
    match reference {
        Some(gref) => repo.set_head(gref.name().unwrap()),
        None => repo.set_head_detached(object.id()),
    }
    .map_err(|e| "Could not set HEAD".to_owned() + e.message())?;
    Ok(())
}

pub struct Emscripten {
    pub binaryen_path: Option<PathBuf>,
    pub emscripten_path: PathBuf,
    pub emscripten_llvm_path: PathBuf,
}

pub fn initialize_emscripten(
    use_system_emscripten: bool,
    targeting_webasm: bool,
) -> Option<Emscripten> {
    let emscripten = EmscriptenPackage {
        git: "https://github.com/emscripten-core/emsdk.git".to_owned(),
        rev: "main".to_owned(),
        sdk_version: "2.0.9".to_owned(),
    };

    let emscripten_root = std::path::Path::new("F:\\Repos\\emsdk_my_cargo_web");

    setup_emscripten(&emscripten, &emscripten_root).unwrap();
    check_emscripten();

    let emscripten_path = emscripten_root.join("emscripten");
    let emscripten_llvm_path = emscripten_root.join("emscripten-fastcomp");

    Some(Emscripten {
        binaryen_path: None,
        emscripten_path,
        emscripten_llvm_path,
    })
}
