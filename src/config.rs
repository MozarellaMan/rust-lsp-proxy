use clap::arg_enum;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Language Server Proxy", about = "A language server proxy")]
pub struct LSArgs {
    #[structopt(short, long, env)]
    pub codebase_path: String,

    #[structopt(short, long, default_value = "8001", env)]
    pub port: i32,

    #[structopt(short, long, env)]
    pub language: Lang,

    #[structopt(short = "s", long, env)]
    pub lang_server_path: String,
}

arg_enum! {
    #[derive(Debug)]
    pub enum Lang {
        Java,
        C
    }
}
