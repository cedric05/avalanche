use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(default_value = "config/config.json5")]
    pub config: String,

    #[clap(default_value_t = 3000)]
    pub port: u16,
}
