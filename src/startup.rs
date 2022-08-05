use axum::handler::Handler;
use axum::routing::get;
use axum::{Extension, Router};

use rusqlite_migration::{Migrations, M};
use tower::builder::ServiceBuilder;
use tower_http::compression::CompressionLayer;

use super::routes::fallback::handler_404;
use crate::configuration::get_configuration;

use crate::routes::{index, login, logout};
use crate::telemetry::{init_telemetry, setup_telemetry};

#[derive(Clone)]
pub struct SessionCookieName(pub String);

pub async fn run() {
    init_telemetry();
    let configuration = get_configuration().expect("Failed to read configuration");
    let hmac_secret = cookie::Key::from(configuration.hmac_secret.as_bytes());
    let session_cookie_name = SessionCookieName(configuration.session_cookie_name);
    let addr = format!("{}:{}", configuration.listen, configuration.port)
        .parse()
        .unwrap();

    let migrations = Migrations::new(vec![M::up(include_str!("sql/00-create_tables.up.sql"))
        .down(include_str!("sql/00-create_tables.down.sql"))]);
    let cfg = deadpool_sqlite::Config::new(configuration.database.connection);
    let pool = cfg.create_pool(deadpool_sqlite::Runtime::Tokio1).unwrap();
    pool.get()
        .await
        .unwrap()
        .interact(move |conn| {
            // conn.pragma_update(None, "journal_mode", &"WAL")
            migrations.to_latest(conn).unwrap();
        })
        .await
        .unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/", get(index::handler))
        .route("/login", get(login::handler))
        .route("/logout", get(logout::handler))
        .fallback(handler_404.into_service());

    let app = app.layer(
        ServiceBuilder::new()
            .layer(CompressionLayer::new().gzip(true).deflate(true).br(true))
            .layer(Extension(pool))
            .layer(Extension(hmac_secret))
            .layer(Extension(session_cookie_name)),
    );

    let app = setup_telemetry(app);

    // run it

    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
