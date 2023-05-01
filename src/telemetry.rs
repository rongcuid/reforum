use std::os::raw::c_int;
use poem::{Body, EndpointExt, Request, Route};

use tokio::task::JoinHandle;
use tower_http::{
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::*;
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*};

fn rusqlite_log(error_code: c_int, msg: &str) {
    error!("SQLite Error {}: {}", error_code, msg);
}

pub fn init_telemetry() {
    color_eyre::install().unwrap();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::CLOSE))
        .init();
    unsafe {
        rusqlite::trace::config_log(Some(rusqlite_log)).unwrap();
    }
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
