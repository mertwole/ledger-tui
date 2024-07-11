use log::LevelFilter;
use pretty_env_logger::env_logger::fmt::{Target, TimestampPrecision};
use std::fs::File;

mod app;
mod device;
mod window;

mod api;
mod device_selection;

use app::App;

#[tokio::main]
async fn main() {
    let log_file = File::create("./log").unwrap();
    pretty_env_logger::formatted_timed_builder()
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .target(Target::Pipe(Box::from(log_file)))
        .filter(None, LevelFilter::Info)
        .init();

    let ledger_api = api::ledger::Ledger {};

    let app = App::new(ledger_api).await;
    app.run().await;
}
