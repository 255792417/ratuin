use clap::Parser;

use ratuin::ratuin::{app, cli::Cli};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    app::run(cli)
}
