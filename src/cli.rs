use clap::Parser;

#[derive(Parser)]
pub (crate) struct Args {
    #[clap(short, long, default_value = "config/config.json5")]
    pub (crate) config: String,

    #[cfg(feature = "sql")]
    #[clap(short, long)]
    pub (crate) db: Option<String>,

    #[clap(short, long, default_value = "localhost:3000")]
    pub (crate) addr: String,
}
