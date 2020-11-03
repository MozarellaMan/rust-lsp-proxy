use std::{process::{Child, Command}};
use crate::config::Lang;


const TEST_JAVA_SERVER_PATH: &str = 
"/home/ayomide/Development/LanguageServers/Java/eclipse.jdt.ls/org.eclipse.jdt.ls.product/target/repository";

pub fn start_server(lang: Lang, file_path: String) -> Option<Child> {
    match lang {
        Lang::Java => {
               Some(Command::new("java")
                .current_dir(TEST_JAVA_SERVER_PATH)
                .arg("-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=1044")
                .arg("-Declipse.application=org.eclipse.jdt.ls.core.id1")
                .arg("-Dosgi.bundles.defaultStartLevel=4")
                .arg("-Declipse.product=org.eclipse.jdt.ls.core.product")
                .arg("-Dlog.level=ALL")
                .arg("-noverify")
                .arg("-Xmx1G")
                .arg("-jar")
                .arg("./plugins/org.eclipse.equinox.launcher_1.5.700.v20200207-2156.jar")
                .arg("-configuration")
                .arg("./config_linux")
                .arg("-data")
                .arg(file_path)
                .arg("--add-modules=ALL-SYSTEM")
                .arg("--add-opens")
                .arg("java.base/java.util=ALL-UNNAMED")
                .arg("--add-opens")
                .arg("java.base/java.lang=ALL-UNNAMED")
                .spawn()
                .expect("failed to execute"))
        },
        Lang::C => None
    }
    
}