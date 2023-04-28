use axum::handler::HandlerWithoutStateExt;
use axum::routing::get;
use axum::{Extension, Router};
use axum_sessions::{async_session, SameSite, SessionLayer};
use rand::Rng;
use std::time::Duration;

use rusqlite_migration::{Migrations, M};
use tower::builder::ServiceBuilder;
use tower_http::compression::CompressionLayer;

use super::routes::fallback::handler_404;
use crate::configuration::{get_configuration, SessionCookieName};

use crate::routes::*;
use crate::telemetry::{init_telemetry, setup_telemetry};

pub async fn run() -> color_eyre::Result<()> {
    init_telemetry();
    let configuration = get_configuration().expect("Failed to read configuration");

    let store = async_session::MemoryStore::new();
    let session_cookie_name = configuration.session_cookie_name;
    let secret = rand::thread_rng().gen::<[u8; 128]>();
    let session_layer = SessionLayer::new(store, &secret)
        .with_cookie_name(session_cookie_name.0)
        .with_same_site_policy(SameSite::Strict)
        .with_http_only(true)
        .with_secure(true)
        .with_session_ttl(Some(Duration::from_secs(60 * 60 * 24 * 30)));

    let addr = format!("{}:{}", configuration.listen, configuration.port).parse()?;

    let migrations = Migrations::new(vec![M::up(include_str!("sql/00-create_tables.up.sql"))
        .down(include_str!("sql/00-create_tables.down.sql"))]);
    let mut db = rusqlite::Connection::open(&configuration.database.connection)?;
    migrations.to_latest(&mut db)?;

    // build our application with a route
    let app = Router::new()
        .route("/", get(index::handler))
        // .route("/login", get(login::get_handler).post(login::post_handler))
        // .route("/logout", get(logout::handler))
        // .route(
        //     "/topics/:id",
        //     get(topics::get_handler).post(topics::post_handler),
        // )
        .fallback(handler_404);

    let app = app.layer(
        ServiceBuilder::new()
            .layer(session_layer)
            .layer(CompressionLayer::new().gzip(true).deflate(true).br(true))
            .layer(Extension(configuration.database)),
    );

    let app = setup_telemetry(app);

    // run it

    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
