use poem::listener::TcpListener;
use poem::middleware::Tracing;
use poem::session::{CookieConfig, CookieSession};
use poem::web::cookie::CookieKey;
use poem::*;
use rand::Rng;
use std::time::Duration;

use tower_http::{
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::*;

use rusqlite_migration::{Migrations, M};

use super::routes::fallback;
use crate::configuration::{get_configuration, SessionCookieName};

use crate::routes::*;
use crate::telemetry::init_telemetry;

pub async fn run() -> color_eyre::Result<()> {
    init_telemetry();
    let configuration = get_configuration().expect("Failed to read configuration");

    let session_cookie_name = configuration.session_cookie_name;

    let addr = format!("{}:{}", configuration.listen, configuration.port);

    let migrations = Migrations::new(vec![M::up(include_str!("sql/00-create_tables.up.sql"))
        .down(include_str!("sql/00-create_tables.down.sql"))]);
    let mut db = rusqlite::Connection::open(&configuration.database.connection)?;
    migrations.to_latest(&mut db)?;

    let app = Route::new()
        .at("/", get(index::handler))
        .with(CookieSession::new(
            CookieConfig::signed(CookieKey::generate())
                .name(session_cookie_name.0)
                .max_age(Duration::from_secs(60 * 60 * 24 * 30)),
        ))
        .with(Tracing)
        .catch_error(fallback::handler_404);
    // run it

    info!("listening on {}", addr);
    Server::new(TcpListener::bind(&addr)).run(app).await?;
    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service())
    //     .await?;
    Ok(())
}
