mod cli;
mod env;
mod reload;
mod scope;
mod util;
mod logging;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    cli::run()
}
