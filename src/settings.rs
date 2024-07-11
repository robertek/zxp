use config::{Config, Environment, File};
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use std::sync::RwLock;

const DEFAULT_CONFIG: &str = "zxp.toml";
const SYSTEM_CONFIG: &str = "/etc/zxp/zxp.toml";
const USER_CONFIG: &str = ".config/zxp/zxp.toml";

lazy_static! {
    static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new());
}


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
    fn new() -> Self {
        let settings: Settings = Default::default();
        settings
    }

    pub fn init(cfgfile: Option<String>) {
        let file = match cfgfile {
            Some(x) => x,
            None => DEFAULT_CONFIG.to_string()
        };

        let mut new_settings = SETTINGS.write().unwrap();
        *new_settings = build_config(&file);
    }

    pub fn gh_repo() -> Result<String, String> {
        match &SETTINGS.read().unwrap().github {
            Some(g) => Ok(g.repo.clone()),
            None => Err(format!("Github config is missing"))
        }
    }

    pub fn gh_key() -> Result<String, String> {
        match &SETTINGS.read().unwrap().github {
            Some(g) => Ok(g.key.clone()),
            None => Err(format!("Github config is missing"))
        }
    }

    pub fn verbosity() -> u8 {
        match SETTINGS.read().unwrap().verbose {
            Some(v) => v,
            None => 0
        }
    }
}

