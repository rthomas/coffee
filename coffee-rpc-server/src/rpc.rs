use coffee_common::coffee::coffee_server::Coffee;
use coffee_common::coffee::{
    AddCoffeeRequest, AddCoffeeResponse, CoffeeItem, ListCoffeeRequest, ListCoffeeResponse,
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

        dbg!("USER ENTRY: {:#?}", &user);

        let resp = RegisterResponse {
            success: true,
            api_key: user.apikey,
        };
        Ok(Response::new(resp))
    }

    async fn add_coffee(
        &self,
        req: Request<AddCoffeeRequest>,
    ) -> Result<Response<AddCoffeeResponse>, Status> {
        dbg!("Adding coffee: {:#?}", &req);

        let api_key = &req.get_ref().api_key;
        let coffee = match &req.get_ref().coffee {
            Some(c) => coffee_common::db::Coffee {
                shots: c.shots,
                utctime: c.utc_time,
            },
            None => {
                return Err(Status::invalid_argument("No coffee provided..."));
            }
        };

        self.db.add_coffee(api_key, &coffee).await?;
        let resp = AddCoffeeResponse { success: true };
        Ok(Response::new(resp))
    }

    async fn list_coffee(
        &self,
        req: Request<ListCoffeeRequest>,
    ) -> Result<Response<ListCoffeeResponse>, Status> {
        dbg!("Listing coffee: {:#?}", &req);

        let api_key = &req.get_ref().api_key;

        let db_coffees = self.db.get_coffees(api_key).await?;

        let mut coffees = Vec::with_capacity(db_coffees.len());

        for c in db_coffees {
            coffees.push(CoffeeItem {
                utc_time: c.utctime,
                shots: c.shots,
            });
        }

        let resp = ListCoffeeResponse { coffees };

        Ok(Response::new(resp))
    }
}
