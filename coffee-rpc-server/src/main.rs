mod rpc;

use coffee_common::coffee::coffee_server::CoffeeServer;
use coffee_common::db::Db;
use rpc::CoffeeService;

use clap::{App, AppSettings, Arg};
use tonic::transport::Server;

static DEFAULT_ADDR: &str = "[::1]:50051";
static DEFAULT_DB: &str = "coffee_db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::ColoredHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("addr")
                .short("a")
                .long("addr")
                .help(
                    format!(
                        "Override the default interface address to bind to: {}",
                        DEFAULT_ADDR
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
        .arg(
            Arg::with_name("db")
                .long("db")
                .help(
                    format!(
                        "Specify the database file location, defaults to: {}",
                        DEFAULT_DB
                    )
                    .as_str(),
                )
                .required(false)
                .global(true),
        )
        .get_matches();

    let addr = matches.value_of("addr").unwrap_or(DEFAULT_ADDR).parse()?;
    let db = matches.value_of("db").unwrap_or(DEFAULT_DB);

    let coffee = CoffeeService::new(Db::new(db).await?);

    Server::builder()
        .add_service(CoffeeServer::new(coffee))
        .serve(addr)
        .await?;

    Ok(())
}
