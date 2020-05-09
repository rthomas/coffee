use coffee_common::coffee::coffee_server::Coffee;
use coffee_common::coffee::{
    AddCoffeeRequest, AddCoffeeResponse, ApiKey, ListCoffeeRequest, ListCoffeeResponse,
    RegisterRequest, RegisterResponse,
};
use coffee_common::db::Db;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
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

        let api_key = &req.get_ref().key.as_ref().unwrap().key;

        println!("COFFEES: {:#?}", self.db.get_coffees(api_key).await?);

        let resp = ListCoffeeResponse { coffees: vec![] };

        Ok(Response::new(resp))
    }
}
