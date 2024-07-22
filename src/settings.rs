use config::{Config, Environment, File};
use serde_derive::Deserialize;
use std::sync::{OnceLock, RwLock};

const DEFAULT_CONFIG: &str = "zxp.toml";
const SYSTEM_CONFIG: &str = "/etc/zxp/zxp.toml";
const USER_CONFIG: &str = ".config/zxp/zxp.toml";


#[derive(Debug, Default, Clone, Deserialize)]
struct Github {
    key: String,
    repo: String,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Settings {
    verbose: Option<u8>,
    github: Option<Github>,
}


fn settings() -> &'static RwLock<Settings> {
    static SETTINGS: OnceLock<RwLock<Settings>> = OnceLock::new();
    SETTINGS.get_or_init(|| RwLock::new(Settings::default()))
}

fn build_config(file: &str) -> Settings {
    // Format the user config in a home directory.
    // Using HOME env variable is not the best and portable approach, but it
    // should be ok for the target use
    let user_config = format!("{}/{}", std::env::var("HOME").unwrap(), USER_CONFIG);

    let s = Config::builder()
        // System config
        .add_source(File::with_name(SYSTEM_CONFIG).required(false))
        // User file
        .add_source(File::with_name(&user_config).required(false))
        // Configuration file
        .add_source(File::with_name(file).required(false))
        // Add in settings from the environment (with a prefix of ZXP_)
        .add_source(Environment::with_prefix("zxp"))
        .build()
        .expect("Config build failed");

    // Deserialize (and thus freeze) the entire configuration
    s.try_deserialize().unwrap()
}

impl Settings {
    pub fn init(cfgfile: Option<String>) {
        let file = cfgfile.unwrap_or(DEFAULT_CONFIG.to_string());

        let mut new_settings = settings().write().unwrap();
        *new_settings = build_config(&file);
    }

    pub fn gh_repo() -> Result<String, String> {
        match &settings().read().unwrap().github {
            Some(g) => Ok(g.repo.clone()),
            None => Err("Github config is missing".to_string())
        }
    }

    pub fn gh_key() -> Result<String, String> {
        match &settings().read().unwrap().github {
            Some(g) => Ok(g.key.clone()),
            None => Err("Github config is missing".to_string())
        }
    }

    pub fn verbosity() -> u8 {
        settings().read().unwrap().verbose.unwrap_or(0)
    }
}

