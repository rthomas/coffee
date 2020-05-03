use crypto::digest::Digest;
use crypto::sha1::Sha1;
use tonic::{transport::Server, Request, Response, Status};

use coffee_common::coffee::coffee_server::{Coffee, CoffeeServer};
use coffee_common::coffee::{
    AddCoffeeRequest, AddCoffeeResponse, ApiKey, CoffeeItem, ListCoffeeRequest, ListCoffeeResponse,
    RegisterRequest, RegisterResponse,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let coffee = CoffeeService::default();
    Server::builder()
        .add_service(CoffeeServer::new(coffee))
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
struct CoffeeService {}

#[tonic::async_trait]
impl Coffee for CoffeeService {
    async fn add_coffee(
        &self,
        req: Request<AddCoffeeRequest>,
    ) -> Result<Response<AddCoffeeResponse>, Status> {
        println!("Adding coffee: {:?}", req);

        let resp = AddCoffeeResponse { success: true };

        Ok(Response::new(resp))
    }

    async fn list_coffee(
        &self,
        req: Request<ListCoffeeRequest>,
    ) -> Result<Response<ListCoffeeResponse>, Status> {
        println!("Listing coffee: {:?}", req);

        let resp = ListCoffeeResponse { coffees: vec![] };

        Ok(Response::new(resp))
    }

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
}
