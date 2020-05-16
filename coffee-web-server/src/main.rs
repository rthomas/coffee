#[macro_use]
extern crate serde_json;

use coffee_common::db::Db;

use actix_web::{get, web, HttpResponse, HttpServer};
use clap::{AppSettings, Arg};
use handlebars::Handlebars;

static DEFAULT_ADDR: &str = "[::1]:8080";
static DEFAULT_DB: &str = "coffee_db";

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("index!")
}

#[get("/c/{api_key}")]
async fn get_coffee(
    db: web::Data<Db>,
    hb: web::Data<Handlebars<'_>>,
    api_key: web::Path<String>,
) -> HttpResponse {
    let key = api_key.to_string();

    println!("Key: {}", &key);

    let coffees = match db.get_coffees(&key).await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error: {}", e));
        }
    };
    let data = json!({
        "api_key": format!("{}", api_key),
        "coffee_count": coffees.len(),
    });

    match hb.render("coffee", &data) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
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
                .takes_value(true)
                .required(false)
                .global(true),
        )
        .arg(
            Arg::with_name("db")
                .short("d")
                .long("db")
                .help(
                    format!(
                        "Specify the database file location, defaults to: {}",
                        DEFAULT_DB
                    )
                    .as_str(),
                )
                .takes_value(true)
                .required(false)
                .global(true),
        )
        .get_matches();
    let addr = matches.value_of("addr").unwrap_or(DEFAULT_ADDR);
    let db_file = matches.value_of("db").unwrap_or(DEFAULT_DB);

    println!("Address: {}", addr);
    println!("Database: {}", db_file);

    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(".html", "./templates")?;
    let hb_ref = web::Data::new(handlebars);

    let db = Db::new(db_file).await?;
    let db_ref = web::Data::new(db);

    HttpServer::new(move || {
        actix_web::App::new()
            .app_data(hb_ref.clone())
            .app_data(db_ref.clone())
            .service(index)
            .service(get_coffee)
    })
    .bind(addr)?
    .run()
    .await?;

    Ok(())
}
