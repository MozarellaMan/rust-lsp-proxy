use clap::arg_enum;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Language Server Proxy", about = "A headless LSP client")]
pub struct LSArgs {
    #[structopt(
        short,
        long,
        default_value = "./tests/example_code_repos/test-java-repo/",
        env
    )]
    pub codebase_path: String,

    #[structopt(short, long, default_value = "8001", env)]
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
