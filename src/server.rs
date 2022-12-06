use payments::{
    pog_lib_server::{PogLib, PogLibServer},
    AddBookRequest, AddBookResponse, Status,
};
use tonic::{transport::Server, Request, Response};

pub mod payments {
    tonic::include_proto!("poglib");
}

#[derive(Debug, Default)]
pub struct PogLibService {}

#[tonic::async_trait]
impl PogLib for PogLibService {
    async fn add_book(
        &self,
        request: Request<AddBookRequest>,
    ) -> Result<Response<AddBookResponse>, tonic::Status> {
        let reply = AddBookResponse {
            status: Status::Ok.into(),
            message: "EVERYTHINHG IS OK".into(),
        };

        println!("ADD_BOOK: {:?}", request.get_ref().name);

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:42069".parse()?;
    let btc_service = PogLibService::default();

    Server::builder()
        .add_service(PogLibServer::new(btc_service))
        .serve(addr)
        .await?;

    Ok(())
}
