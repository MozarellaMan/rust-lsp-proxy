use clap::arg_enum;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Language Server Proxy", about = "A headless LSP client")]
pub struct LSArgs {
    #[structopt(short, long, parse(from_os_str), default_value = "./", env)]
    pub codebase_path: std::path::PathBuf,

    #[structopt(short, long, default_value = "8000", env)]
    pub port: i32,

    #[structopt(short, long, default_value = "Java", env)]
    pub language: Lang,
}

arg_enum! {
    #[derive(Debug)]
    pub enum Lang {
        Java,
        C
    }
}
