use coffee_common::coffee::coffee_client::CoffeeClient;
use coffee_common::coffee::coffee_item::Type;
use coffee_common::coffee::{
    AddCoffeeRequest, ApiKey, CoffeeItem, ListCoffeeRequest, RegisterRequest,
};

use clap::{App, AppSettings, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tonic::Request;

static DEFAULT_SERVER: &str = "[::1]:50051";
static DEFAULT_CONFIG: &str = ".coffee";

#[derive(Debug, Deserialize, Serialize)]
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::ArgRequiredElseHelp)
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
        // TODO: Add List subcommands.
        .subcommand(
            SubCommand::with_name("add").about("Adds a coffee").arg(
                Arg::with_name("AMOUNT")
                    .required(true)
                    .help("The amount of coffee, in shots"),
            ),
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
            SubCommand::with_name("list")
                .about("List the coffees for this registered users, with an optional date")
                .arg(
                    Arg::with_name("DATE")
                        .required(false)
                        .help("A date to list the coffees for."),
                ),
        )
        .get_matches();

    // TODO: Read config from default (or overridden) location
    // TODO: Read in API Key and server defaults if set
    // TODO: if no API key then error out.

    let config = match matches.value_of("config") {
        Some(s) => read_config_if_exists(Path::new(s)),
        None => {
            let mut home = dirs::home_dir().expect("Could not locate a home directory...");
            home.push(DEFAULT_CONFIG);
            read_config_if_exists(home.as_path())
        }
    };

    println!("Config: {:#?}", config);

    let addr = match matches.value_of("server") {
        Some(s) => s,
        None => DEFAULT_SERVER,
    };

    let mut client = CoffeeClient::connect(format!("http://{}", addr)).await?;

    if let Some(_cmd) = matches.subcommand_matches("add") {
        let add_req = Request::new(AddCoffeeRequest {
            key: Some(ApiKey { key: "foo".into() }),
            coffee: Some(CoffeeItem {
                utc_time: 0,
                coffee_type: Type::SingleShot.into(),
            }),
        });
        let resp = client.add_coffee(add_req).await?;
        println!("Response: {:#?}", resp);
    } else if let Some(cmd) = matches.subcommand_matches("register") {
        // TODO: The api key should be mailed, this isn't great - we need a subcommand to take the apikey and write it to the config.
        // For now we'll just write it back to the config.

        let reg_req = Request::new(RegisterRequest {
            email: cmd.value_of("EMAIL").unwrap().into(),
        });

        let resp = client.register(reg_req).await?;
        println!("Register Response: {:#?}", resp);
    } else if let Some(_cmd) = matches.subcommand_matches("list") {
        let list_req = Request::new(ListCoffeeRequest {
            key: Some(ApiKey {
                key: String::from(""),
            }),
            start_utc_time: 0,
            end_utc_time: 0,
        });

        let resp = client.list_coffee(list_req).await?;
        println!("List Response: {:#?}", resp);
    }

    Ok(())
}
