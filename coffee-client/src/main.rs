mod error;

use coffee_common::coffee::coffee_client::CoffeeClient;
use coffee_common::coffee::{AddCoffeeRequest, CoffeeItem, ListCoffeeRequest, RegisterRequest};
use error::ClientError;

use chrono::prelude::*;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use tonic::Request;

static DEFAULT_SERVER: &str = "[::1]:50051";
static DEFAULT_CONFIG: &str = ".coffee";

#[derive(Debug, Default, Deserialize, Serialize)]
struct CoffeeConfig {
    api_key: String,
}

/// The config won't always exist - a helper function to check it for us.
fn read_config_if_exists(cfg: &Path) -> Option<std::io::Result<CoffeeConfig>> {
    if cfg.exists() {
        Some(read_config(cfg))
    } else {
        None
    }
}

fn read_config(cfg: &Path) -> std::io::Result<CoffeeConfig> {
    let reader = BufReader::new(File::open(cfg)?);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn write_config(cfg: &CoffeeConfig) -> std::io::Result<()> {
    let mut cfg_file = dirs::home_dir().expect("Could not locate a home directory...");
    cfg_file.push(DEFAULT_CONFIG);
    let writer = BufWriter::new(File::create(cfg_file)?);
    serde_json::to_writer(writer, cfg)?;
    Ok(())
}

// Gets the API Key from either the args or the config. Args take precedence
// over the config.
fn get_api_key<'a>(
    cfg: &'a Option<CoffeeConfig>,
    args: &'a ArgMatches,
) -> Result<&'a str, ClientError> {
    match args.value_of("key") {
        Some(k) => Ok(k),
        None => match cfg {
            Some(c) => Ok(&c.api_key),
            None => Err(ClientError::NoApiKey),
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let key_arg = Arg::with_name("key")
        .required(false)
        .short("k")
        .long("key")
        .help("Override the API key used");
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .help(
                    format!(
                        "Override the default server to connect to, default is: {}",
                        DEFAULT_SERVER
                    )
                    .as_str(),
                )
                .required(false)
                .global(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Specify a config file")
                .required(false)
                .global(true),
        )
        .subcommand(
            SubCommand::with_name("register")
                .about("Registers an email against an API Key")
                .arg(
                    Arg::with_name("EMAIL")
                        .required(true)
                        .help("An email address (to be verified against)"),
                ),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Adds a coffee")
                .arg(&key_arg)
                .arg(
                    Arg::with_name("AMOUNT")
                        .required(true)
                        .help("The amount of coffee, in shots"),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List the coffees for this registered users, with an optional date")
                .arg(&key_arg)
                .arg(
                    Arg::with_name("DATE")
                        .required(false)
                        .help("A date to list the coffees for."),
                ),
        )
        .get_matches();

    let config = match matches.value_of("config") {
        Some(s) => read_config_if_exists(Path::new(s)),
        None => {
            let mut home = dirs::home_dir().expect("Could not locate a home directory...");
            home.push(DEFAULT_CONFIG);
            read_config_if_exists(home.as_path())
        }
    }
    .and_then(|f| match f {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            eprintln!("Could not read config file: {}", e);
            None
        }
    });

    let addr = matches.value_of("server").unwrap_or(DEFAULT_SERVER);
    let mut client = CoffeeClient::connect(format!("http://{}", addr)).await?;

    if let Some(cmd) = matches.subcommand_matches("register") {
        // TODO: The api key should be mailed, this isn't great - we need a subcommand to take the apikey and write it to the config.
        // For now we'll just write it back to the config.

        let reg_req = Request::new(RegisterRequest {
            email: cmd.value_of("EMAIL").unwrap_or("").into(),
        });

        let resp = client.register(reg_req).await?;
        let resp = resp.get_ref();

        if resp.success {
            // Make sure we only update the api key here if config already
            // exists - so use the existing one or get a default instance of it.
            let mut config = config.or(Some(CoffeeConfig::default())).unwrap();
            config.api_key = resp.api_key.clone();
            write_config(&config)?;
            println!("Config updates.");
        } else {
            eprintln!("Server error when registering.");
            return Err(ClientError::RegistrationError);
        }
    } else if let Some(cmd) = matches.subcommand_matches("add") {
        let api_key = get_api_key(&config, cmd)?;

        let shots = match cmd.value_of("AMOUNT").unwrap_or("").parse() {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Cannot convert argument to number: {}", e);
                return Err(ClientError::BadArgument);
            }
        };

        // Seconds from unix epoch.
        let utc_time = Utc::now().timestamp();

        let add_req = Request::new(AddCoffeeRequest {
            api_key: api_key.into(),
            coffee: Some(CoffeeItem { utc_time, shots }),
        });
        let resp = client.add_coffee(add_req).await?;

        if resp.get_ref().success {
            println!("Done!");
        } else {
            return Err(ClientError::AddFailed);
        }
    } else if let Some(cmd) = matches.subcommand_matches("list") {
        let api_key = get_api_key(&config, cmd)?;

        let list_req = Request::new(ListCoffeeRequest {
            api_key: api_key.into(),
            start_utc_time: 0,
            end_utc_time: 0,
        });

        let resp = client.list_coffee(list_req).await?;

        if !&resp.get_ref().coffees.is_empty() {
            let mut daily_coffees = HashMap::new();
            for coffee in &resp.get_ref().coffees {
                // Group the coffees by date.
                let t = Utc.timestamp(coffee.utc_time, 0).with_timezone(&Local);
                let coffees = daily_coffees.entry(t.date()).or_insert_with(|| Vec::new());
                coffees.push(coffee);
            }

            for (date, coffees) in daily_coffees {
                let mut acc = 0;
                println!("{:20}", date);
                for c in coffees {
                    let t = Utc.timestamp(c.utc_time, 0).with_timezone(&Local);
                    acc += c.shots;
                    println!("{:16}{:10}: {}", "", t.time(), c.shots);
                }
                println!("{:26}{}", "Daily Total:", acc);
            }
        } else {
            println!("Nothing found :(");
        }
    }

    Ok(())
}
