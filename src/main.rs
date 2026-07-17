use clap::Parser;
use rustytherobot::cli::Cli;
use rustytherobot::run::run;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

fn main() {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.´
        .with_env_filter("info")
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();
    info!("main started");
    run(cli);
}
