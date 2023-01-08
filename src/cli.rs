use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value = "config/config.json5")]
    pub config: String,

    #[cfg(feature = "sql")]
    #[clap(short, long)]
    pub db: Option<String>,

    #[clap(short, long, default_value_t = 3000)]
    pub port: u16,
}
