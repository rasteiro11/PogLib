use database::database::get_connection;
use diesel::prelude::*;
use diesel::{MysqlConnection, QueryDsl, RunQueryDsl};
use dotenvy::dotenv;
use models::models::{Book, NewBook};
use payments::{
    pog_lib_server::{PogLib, PogLibServer},
    AddBookRequest, AddBookResponse, Status,
};
use std::env;
use tonic::{transport::Server, Request, Response};

pub mod payments {
    tonic::include_proto!("poglib");
}

pub mod database;
pub mod models;
pub mod schema;

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

pub fn add_book(conn: &mut MysqlConnection, name: &str) {
    use schema::books::dsl::books;
    let book = NewBook { name };

    diesel::insert_into(books)
        .values(&book)
        .execute(conn)
        .expect("Error saving book");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:42069".parse()?;
    let btc_service = PogLibService::default();

    Server::builder()
        .add_service(PogLibServer::new(btc_service))
        .serve(addr)
        .await?;

    dotenv().ok();

    let url = env::var("DATABASE_URL").expect("DATABASE_URL is not present in .env file");
    let mut conn = get_connection(&url);

    let rows_affected = add_book(&mut conn, "INTRESTING BOOK");

    println!("AFFECTED ROWS: {}", rows_affected);

    Ok(())
}
