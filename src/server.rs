use crate::models::models::Book;
use diesel::*;
use diesel::{prelude::*, sql_types::Text};
use dotenvy::dotenv;
use payments::pog_lib_server::PogLib;
use payments::{pog_lib_server::PogLibServer, AddBookRequest, AddBookResponse, Status};
use std::borrow::{Borrow, BorrowMut};
use std::env;
use tonic::codegen::Body;
use tonic::transport::Server;
use tonic::{Request, Response};

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
        let conn: &mut MysqlConnection = &mut connection();

        let bs = sql_query("INSERT INTO books (name) VALUES (?)")
            .bind::<Text, _>(&request.get_ref().name)
            .execute(conn)
            .unwrap();

        let added_book = sql_query("SELECT * FROM books ORDER BY id DESC")
            .load::<Book>(conn)
            .unwrap();

        let reply = AddBookResponse {
            status: Status::Ok.into(),
            message: format!("New book was added and it's id is {}", added_book[0].id),
        };

        Ok(Response::new(reply))
    }
}

fn connection() -> MysqlConnection {
    dotenvy::dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in order to run unit tests");
    MysqlConnection::establish(&database_url).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:42069".parse()?;
    let books_service = PogLibService::default();

    Server::builder()
        .add_service(PogLibServer::new(books_service))
        .serve(addr)
        .await?;

    Ok(())
}
