use coffee_common::coffee::coffee_server::{Coffee, CoffeeServer};
use coffee_common::coffee::{
    AddCoffeeRequest, AddCoffeeResponse, ApiKey, ListCoffeeRequest, ListCoffeeResponse,
    RegisterRequest, RegisterResponse,
};
use coffee_common::db::Db;

use clap::{App, AppSettings, Arg};
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use tonic::{transport::Server, Request, Response, Status};

static DEFAULT_ADDR: &str = "[::1]:50051";
static DEFAULT_DB: &str = "coffee_db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::ArgRequiredElseHelp)
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

    let coffee = CoffeeService {
        db: Db::new(db).await?,
    };

    Server::builder()
        .add_service(CoffeeServer::new(coffee))
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug)]
struct CoffeeService {
    db: Db,
}

#[tonic::async_trait]
impl Coffee for CoffeeService {
    async fn register(
        &self,
        req: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        // TODO: The actual registration flow - i.e. check if registered and return same api key, otherwise just return a new api key for them for now and store in the db.

        // For now we will just use a sha1 hash of the email... not ideal but fine for now.

        let mut hasher = Sha1::new();
        hasher.input_str(&req.get_ref().email);

        let resp = RegisterResponse {
            success: true,
            key: Some(ApiKey {
                key: hasher.result_str(),
            }),
        };
        Ok(Response::new(resp))
    }

    async fn add_coffee(
        &self,
        req: Request<AddCoffeeRequest>,
    ) -> Result<Response<AddCoffeeResponse>, Status> {
        println!("Adding coffee: {:#?}", req);

        let resp = AddCoffeeResponse { success: true };

        Ok(Response::new(resp))
    }

    async fn list_coffee(
        &self,
        req: Request<ListCoffeeRequest>,
    ) -> Result<Response<ListCoffeeResponse>, Status> {
        println!("Listing coffee: {:#?}", req);

        let api_key = &req.get_ref().key;

        let key = "";

        println!("COFFEES: {:#?}", self.db.get_coffees(key).await.unwrap());

        let resp = ListCoffeeResponse { coffees: vec![] };

        Ok(Response::new(resp))
    }
}
