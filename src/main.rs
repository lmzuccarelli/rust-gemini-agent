use crate::cli::schema::Cli;
use crate::config::load::{ConfigInterface, ImplConfigInterface};
use crate::handlers::process::{Agent, AgentInterface};
use clap::Parser;
use custom_logger as log;

mod cli;
mod config;
mod error;
mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = args.config;
    let impl_config = ImplConfigInterface {};

    // setup logging
    log::Logging::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("log should initialize");

    // read and parse config
    let params = impl_config.read(config);
    if params.is_err() {
        log::error!("{}", params.err().unwrap());
        std::process::exit(1);
    }

    let level = match params.as_ref().unwrap().log_level.as_str() {
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        &_ => log::LevelFilter::Info,
    };

    // override level if other than info
    if level == log::LevelFilter::Debug || level == log::LevelFilter::Trace {
        let _ = log::Logging::new().with_level(level).init();
    }

    log::info!("application : {}", env!("CARGO_PKG_NAME"));
    log::info!("author      : {}", env!("CARGO_PKG_AUTHORS"));
    log::info!("version     : {}", env!("CARGO_PKG_VERSION"));

    let res = Agent::execute(params.unwrap(), args.key).await;
    match res {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            println!("{}", e.to_string());
            println!("exit => 2")
        }
    }
    Ok(())
}
