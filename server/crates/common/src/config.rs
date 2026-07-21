//! Runtime configuration, loaded from environment variables with the same
//! names Immich uses (IMMICH_*/DB_*) so existing deployments can point at
//! Domus without changes, plus DOMUS_* overrides.

use figment::providers::{Env, Serialized};
use figment::Figment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// HTTP listen host.
    #[serde(default = "default_host")]
    pub host: String,
    /// HTTP listen port (Immich default: 2283).
    #[serde(default = "default_port")]
    pub port: u16,
    /// Machine-learning service listen port. Domus defaults to 3004 so it can
    /// run alongside the official Immich ML service on 3003.
    #[serde(default = "default_ml_port")]
    pub ml_port: u16,
    /// Root of the media directory (Immich mounts this at /data).
    #[serde(default = "default_media_location")]
    pub media_location: String,
    /// Optional source media root used by rows imported from/native to Immich.
    /// When set, paths stored under this root are served from `media_location`.
    #[serde(default)]
    pub original_media_location: Option<String>,
    pub database: DatabaseConfig,
    #[serde(default)]
    pub workers: WorkerConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_url")]
    pub url: String,
    #[serde(default = "default_db_pool")]
    pub max_connections: u32,
    /// Run Domus-owned migrations on startup. The SQLite deployment normally
    /// uses an already-created database from scripts/migrate_pg_to_sqlite.py.
    #[serde(default)]
    pub run_migrations: bool,
}

/// Which worker groups this process runs. Mirrors IMMICH_WORKERS_INCLUDE /
/// IMMICH_WORKERS_EXCLUDE: a single binary can run "api", "microservices"
/// (background jobs) or both.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkerConfig {
    #[serde(default = "default_true")]
    pub api: bool,
    #[serde(default = "default_true")]
    pub microservices: bool,
    #[serde(default = "default_true")]
    pub ml: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            api: true,
            microservices: true,
            ml: true,
        }
    }
}

impl Config {
    pub fn load() -> crate::Result<Self> {
        let figment = Figment::from(Serialized::defaults(Config::default()))
            .merge(Env::prefixed("DOMUS_").split("__"))
            .join(Env::raw().only(&["HOST", "PORT"]));
        let mut config: Config = figment
            .extract()
            .map_err(|e| crate::Error::Config(e.to_string()))?;
        config.apply_immich_env();
        Ok(config)
    }

    /// Honour the subset of IMMICH_* / DB_* environment variables that map
    /// onto Domus settings, for drop-in compatibility.
    fn apply_immich_env(&mut self) {
        if let Ok(v) = std::env::var("IMMICH_PORT") {
            if let Ok(port) = v.parse() {
                self.port = port;
            }
        }
        if let Ok(v) = std::env::var("IMMICH_MACHINE_LEARNING_PORT") {
            if let Ok(port) = v.parse() {
                self.ml_port = port;
            }
        }
        if let Ok(v) = std::env::var("PORT") {
            if let Ok(port) = v.parse() {
                self.port = port;
            }
        }
        if let Ok(v) = std::env::var("HOST") {
            self.host = v;
        }
        if let Ok(v) = std::env::var("IMMICH_MEDIA_LOCATION") {
            self.media_location = v;
        }
        if let Ok(v) = std::env::var("DOMUS_ORIGINAL_MEDIA_LOCATION") {
            self.original_media_location = (!v.is_empty()).then_some(v);
        }
        if let Ok(v) = std::env::var("DB_URL") {
            self.database.url = v;
        }
        if let Ok(v) = std::env::var("DOMUS_DATABASE_RUN_MIGRATIONS") {
            self.database.run_migrations = parse_bool(&v);
        }
    }
}

fn default_host() -> String {
    "0.0.0.0".into()
}
fn default_port() -> u16 {
    2283
}
fn default_ml_port() -> u16 {
    3004
}
fn default_media_location() -> String {
    "/data".into()
}
fn default_db_url() -> String {
    "sqlite:///opt/usr/local/domus/data/domus.sqlite3".into()
}
fn default_db_pool() -> u32 {
    10
}
fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            ml_port: default_ml_port(),
            media_location: default_media_location(),
            original_media_location: None,
            database: DatabaseConfig {
                url: default_db_url(),
                max_connections: default_db_pool(),
                run_migrations: false,
            },
            workers: WorkerConfig::default(),
        }
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
