use crate::config::{Lang, LsArgs};
use std::{env, fs::read_dir, path::Path, process::Stdio};
use structopt::StructOpt;
use tokio::process::{Child, Command};

/// Runs a language server child process, according to the language passed into the function
pub fn start_lang_server(lang: Lang, temp_files_path: &Path) -> Option<Child> {
    match lang {
        Lang::Java => java_server(&temp_files_path),
        Lang::C => None,
        Lang::Custom => custom_config_server(),
    }
}

/// Runs a language server child process, given *exact* commandline arguments
pub fn custom_config_server() -> Option<Child> {
    let args = LsArgs::from_args();
    let path = Path::new(&args.lang_server_path);
    let custom_lang_server_cmd = args.custom_lang_server_cmd;

    if let Some(custom_lang_server_cmd) = custom_lang_server_cmd {
        Some(
            Command::new(custom_lang_server_cmd)
                .current_dir(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("unable to spawn language server process with custom command"),
        )
    } else {
        None
    }
}

/// Starts a JDT language server process
fn java_server(temp_files_path: &Path) -> Option<Child> {
    let args = LsArgs::from_args();
    // need to find a specific jar file for launching the jdt server
    let path = Path::new(&args.lang_server_path);
    let plugins_dir = path.join("plugins");
    let mut launcher_name: Option<String> = None;
    let config: Option<&str> = match env::consts::OS {
        "windows" => Some("./config_win"),
        "linux" => Some("./config_linux"),
        "macos" => Some("./config_mac"),
        _ => None,
    };

    if let Ok(dirs) = read_dir(plugins_dir) {
        dirs.into_iter().flatten().for_each(|path| {
            if path
                .file_name()
                .to_string_lossy()
                .starts_with("org.eclipse.equinox.launcher_")
            {
                launcher_name = Some(path.file_name().to_string_lossy().to_string())
            }
        });
    }

    if let (Some(launcher_name), Some(config)) = (launcher_name, config) {
        let launcher_path = format!("./plugins/{}", launcher_name);
        Some(
            Command::new("java")
                .current_dir(args.lang_server_path)
                .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=1044")
                .arg("-Declipse.application=org.eclipse.jdt.ls.core.id1")
                .arg("-Dosgi.bundles.defaultStartLevel=4")
                .arg("-Declipse.product=org.eclipse.jdt.ls.core.product")
                .arg("-Dlog.level=ALL")
                .arg("-noverify")
                .arg("-Xmx1G")
                .arg("-jar")
                .arg(launcher_path)
                .arg("-configuration")
                .arg(config)
                .arg("-data")
                .arg(temp_files_path)
                .arg("--add-modules=ALL-SYSTEM")
                .arg("--add-opens")
                .arg("java.base/java.util=ALL-UNNAMED")
                .arg("--add-opens")
                .arg("java.base/java.lang=ALL-UNNAMED")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("unable to spawn java language server process"),
        )
    } else {
        None
    }
}
