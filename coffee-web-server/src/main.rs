#[macro_use]
extern crate tower_web;

use clap::{App, AppSettings, Arg};

use serde::Serialize;
use tower_web::view::Handlebars;
use tower_web::Response;
use tower_web::ServiceBuilder;

static DEFAULT_ADDR: &str = "[::1]:8080";

#[derive(Clone, Debug)]
struct HtmlResource;

#[derive(Response)]
struct CoffeeResponse;

impl_web! {
    impl HtmlResource{
        #[get("/")]
        #[content_type("html")]
        #[web(template = "index")]
        fn get_index(&self) -> Result<CoffeeResponse, ()> {
            Ok(CoffeeResponse{})
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .get_matches();
    let addr = matches.value_of("addr").unwrap_or(DEFAULT_ADDR).parse()?;

    ServiceBuilder::new()
        .resource(HtmlResource)
        .serializer(Handlebars::new())
        .run(&addr)?;

    Ok(())
}
