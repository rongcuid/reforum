use axum::handler::Handler;
use axum::routing::get;
use axum::{Extension, Router};

use migration::{Migrator, MigratorTrait};

use sea_orm::ConnectOptions;
use secrecy::Secret;
use tower::builder::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tracing::log::LevelFilter;

use super::routes::fallback::handler_404;
use crate::configuration::get_configuration;

use crate::routes::index;
use crate::telemetry::{init_telemetry, setup_telemetry};

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

#[derive(Clone)]
pub struct SessionCookieName(pub String);

pub async fn run() {
    init_telemetry();
    let configuration = get_configuration().expect("Failed to read configuration");
    let hmac_secret = HmacSecret(Secret::new(configuration.hmac_secret));
    let session_cookie_name = SessionCookieName(configuration.session_cookie_name);
    let addr = format!("{}:{}", configuration.listen, configuration.port)
        .parse()
        .unwrap();

    let mut connection_options = ConnectOptions::from(&configuration.database.connection);
    connection_options.sqlx_logging_level(LevelFilter::Debug);

    let db = sea_orm::Database::connect(connection_options)
        .await
        .unwrap();
    Migrator::up(&db, None).await.unwrap();

    // build our application with a route
    let app = Router::new()
        .route("/", get(index::handler))
        .fallback(handler_404.into_service());

    let app = app.layer(
        ServiceBuilder::new()
            .layer(CompressionLayer::new().gzip(true).deflate(true).br(true))
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
