use clap::arg_enum;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Language Server Proxy", about = "A language server proxy")]
pub struct LsArgs {
    #[structopt(short, long, env)]
    pub codebase_path: String,

    #[structopt(short, long, default_value = "8001", env)]
    pub port: i32,

    #[structopt(short, long, env)]
    pub language: Lang,

    #[structopt(short = "s", long, env)]
    pub lang_server_path: String,

    #[structopt(short = "d", long, env, required_if("language", "Custom"))]
    pub custom_lang_server_cmd: Option<String>,
}

arg_enum! {
    #[derive(Debug)]
    pub enum Lang {
        Java,
        C,
        Custom
    }
}

pub fn get_ls_args() -> LsArgs {
    LsArgs::from_args()
}
