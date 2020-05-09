use coffee_common::coffee::coffee_server::Coffee;
use coffee_common::coffee::{
    AddCoffeeRequest, AddCoffeeResponse, ApiKey, ListCoffeeRequest, ListCoffeeResponse,
    RegisterRequest, RegisterResponse,
};
use coffee_common::db::Db;

use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct CoffeeService {
    db: Db,
}

impl CoffeeService {
    pub fn new(db: Db) -> Self {
        CoffeeService { db }
    }
}

#[tonic::async_trait]
impl Coffee for CoffeeService {
    async fn register(
        &self,
        req: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let email = &req.get_ref().email;

        let user = self.db.register_user(email).await?;

        println!("USER ENTRY: {:#?}", user);

        let resp = RegisterResponse {
            success: true,
            key: Some(ApiKey { key: user.apikey }),
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

        let api_key = &req.get_ref().key.as_ref().unwrap().key;

        println!("COFFEES: {:#?}", self.db.get_coffees(api_key).await?);

        let resp = ListCoffeeResponse { coffees: vec![] };

        Ok(Response::new(resp))
    }
}
