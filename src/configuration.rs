use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub session_cookie_name: SessionCookieName,
    pub database: SQLite3Settings,
    pub listen: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct SQLite3Settings {
    pub connection: String,
}

impl SQLite3Settings {
    pub fn connect(&self) -> Result<rusqlite::Connection, rusqlite::Error> {
        rusqlite::Connection::open(&self.connection)
    }
}

#[derive(Deserialize, Clone)]
pub struct SessionCookieName(pub String);

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.toml",
            config::FileFormat::Toml,
        ))
        .build()?;
    settings.try_deserialize()
}
