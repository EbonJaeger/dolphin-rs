use std::error::Error;

mod cli;
mod config;
mod discord;
mod listener;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pipeline;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::process().await {
        report_error(e);
        std::process::exit(1);
    }
}

fn report_error(err: cli::Error) {
    let sources = sources(&err);
    let error = sources.join(": ");
    eprintln!("Error: {error}");
}

fn sources(error: &cli::Error) -> Vec<String> {
    let mut sources = vec![error.to_string()];
    let mut source = error.source();

    while let Some(error) = source.take() {
        sources.push(error.to_string());
        source = error.source();
    }

    sources
}
