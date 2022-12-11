use crate::models::models::Book;
use diesel::sql_types::Integer;
use diesel::*;
use diesel::{prelude::*, sql_types::Text};
use payments::pog_lib_server::PogLib;
use payments::{pog_lib_server::PogLibServer, AddBookRequest, AddBookResponse, Status};
use payments::{DeleteBookRequest, ListBooksPagesRequest, UpdateBookRequest, UpdateBookResponse};
use payments::{DeleteBookResponse, ListBooksPagesResponse};
use std::{env, vec};
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

    async fn list_books(
        &self,
        request: Request<ListBooksPagesRequest>,
    ) -> Result<Response<ListBooksPagesResponse>, tonic::Status> {
        let conn: &mut MysqlConnection = &mut connection();

        let n_books = sql_query("SELECT * FROM books WHERE name = ?")
            .bind::<Text, _>(&request.get_ref().name)
            .execute(conn)
            .unwrap() as i32;

        let offset = (request.get_ref().page - 1) * request.get_ref().per_page;
        let pages = n_books / request.get_ref().per_page;

        let books =
            sql_query("SELECT * FROM books WHERE name = ?  ORDER BY id DESC LIMIT ? OFFSET ?")
                .bind::<Text, _>(&request.get_ref().name)
                .bind::<Integer, _>(request.get_ref().per_page)
                .bind::<Integer, _>(offset)
                .load::<Book>(conn)
                .unwrap();

        let mut bs = vec![];
        for book in books {
            bs.push(payments::Book {
                id: book.id,
                name: book.name,
            })
        }

        let reply = ListBooksPagesResponse {
            page: request.get_ref().page,
            pages,
            total: n_books,
            status: Status::Ok.into(),
            message: "".into(),
            data: bs,
        };

        Ok(Response::new(reply))
    }

    async fn delete_book_by_id(
        &self,
        request: Request<DeleteBookRequest>,
    ) -> Result<Response<DeleteBookResponse>, tonic::Status> {
        let conn: &mut MysqlConnection = &mut connection();

        let deleted_book = sql_query("SELECT * FROM books WHERE id = ?")
            .bind::<Integer, _>(&request.get_ref().id)
            .load::<Book>(conn)
            .unwrap();

        let rows_affected = sql_query("DELETE FROM books WHERE id = ?")
            .bind::<Integer, _>(&request.get_ref().id)
            .execute(conn)
            .unwrap();

        let reply = DeleteBookResponse {
            status: Status::Ok.into(),
            message: format!("Deleted book with id: {}", deleted_book[0].id),
            book: Some(payments::Book {
                id: deleted_book[0].id,
                name: deleted_book[0].name.to_string(),
            }),
        };

        Ok(Response::new(reply))
    }

    async fn update_book_by_id(
        &self,
        request: Request<UpdateBookRequest>,
    ) -> Result<Response<UpdateBookResponse>, tonic::Status> {
        let conn: &mut MysqlConnection = &mut connection();

        let deleted_book = sql_query("SELECT * FROM books WHERE id = ?")
            .bind::<Integer, _>(&request.get_ref().id)
            .load::<Book>(conn)
            .unwrap();

        let rows_affected = sql_query("UPDATE books SET name = ? WHERE id = ?")
            .bind::<Text, _>(&request.get_ref().name)
            .bind::<Integer, _>(&request.get_ref().id)
            .execute(conn)
            .unwrap();

        let reply = UpdateBookResponse {
            status: Status::Ok.into(),
            message: format!("Updated book with id: {}", deleted_book[0].id),
            book: Some(payments::Book {
                id: deleted_book[0].id,
                name: request.get_ref().name.to_string(),
            }),
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
