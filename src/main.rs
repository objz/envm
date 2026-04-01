mod cli;
mod completions;
mod env;
mod logging;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    cli::run()
}
