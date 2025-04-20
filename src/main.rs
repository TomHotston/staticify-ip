use clap::Parser;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

const FILE_NAME: &str = "./staticify-ip.toml";

#[derive(Serialize, Deserialize, PartialEq, Clone)]
struct Config {
    token: String,
    website: String,
    zone_id: String,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets alternative location for the config file
    #[arg(short, long, value_name = "FILE")]
    config_file: Option<PathBuf>,
}

impl Config {
    fn new(path: &Path) -> Self {
        let default_file = Self {
            token: String::from("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
            website: String::from("test.example.com"),
            zone_id: String::from("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
        };

        let content = match fs::read_to_string(path).ok() {
            Some(c) => match toml::from_str(&c) {
                Ok(t) => t,
                Err(_) => {
                    error!("TOML file is not configured correctly");
                    panic!("TOML file is not configured correctly");
                }
            },
            _ => default_file.clone(),
        };
        if content == default_file {
            info!("TOML file has not been configured");
            let toml_file = toml::to_string(&content).unwrap();
            fs::write(path, toml_file).unwrap();
            exit(1);
        }

        content
    }
}

fn main() {
    env_logger::init();
    debug!("logger initialised");

    let cli = Cli::parse();

    let path = match cli.config_file {
        Some(path) => path,
        _ => PathBuf::from(FILE_NAME),
    };

    let config = Config::new(&path);

    let server_config =
        staticify_ip::ServerConfigurator::new(&config.token, &config.website, &config.zone_id);
    staticify_ip::configure(server_config).unwrap();
}
