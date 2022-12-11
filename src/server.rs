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

        let rows_affected = sql_query("INSERT INTO books (name) VALUES (?)")
            .bind::<Text, _>(&request.get_ref().name)
            .execute(conn);

        if let Err(err) = rows_affected {
            let reply = AddBookResponse {
                status: Status::Error.into(),
                message: format!("Add book failed with error: {}", err),
            };

            return Ok(Response::new(reply));
        }

        let added_book = sql_query("SELECT * FROM books ORDER BY id DESC").load::<Book>(conn);
        if let Err(err) = added_book {
            let reply = AddBookResponse {
                status: Status::Error.into(),
                message: format!("Add book failed with error: {}", err),
            };

            return Ok(Response::new(reply));
        }

        let added_book = added_book.unwrap();
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

        let mut n_books;
        let mut books;

        if request.get_ref().name.is_empty() {
            n_books = sql_query("SELECT * FROM books")
                .bind::<Text, _>(&request.get_ref().name)
                .execute(conn);
        } else {
            n_books = sql_query("SELECT * FROM books WHERE name = ?")
                .bind::<Text, _>(&request.get_ref().name)
                .execute(conn);
        }

        if let Err(err) = n_books {
            let reply = ListBooksPagesResponse {
                data: vec![],
                page: request.get_ref().page,
                status: Status::Error.into(),
                total: 0,
                pages: 0,
                message: format!("List Books Failed with error: {}", err),
            };

            return Ok(Response::new(reply));
        }

        let n_books = n_books.unwrap() as i32;
        let offset = (request.get_ref().page - 1) * request.get_ref().per_page;
        let pages = n_books / request.get_ref().per_page;

        if request.get_ref().name.is_empty() {
            books = sql_query("SELECT * FROM books ORDER BY id DESC LIMIT ? OFFSET ?")
                .bind::<Integer, _>(request.get_ref().per_page)
                .bind::<Integer, _>(offset)
                .load::<Book>(conn);
        } else {
            books =
                sql_query("SELECT * FROM books WHERE name = ? ORDER BY id DESC LIMIT ? OFFSET ?")
                    .bind::<Text, _>(&request.get_ref().name)
                    .bind::<Integer, _>(request.get_ref().per_page)
                    .bind::<Integer, _>(offset)
                    .load::<Book>(conn);
        }

        if let Err(err) = books {
            let reply = ListBooksPagesResponse {
                data: vec![],
                page: request.get_ref().page,
                status: Status::Error.into(),
                total: 0,
                pages: 0,
                message: format!("List Books Failed with error: {}", err),
            };

            return Ok(Response::new(reply));
        }

        let books = books.unwrap();
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
            .load::<Book>(conn);

        if let Err(err) = deleted_book {
            let reply = DeleteBookResponse {
                status: Status::Error.into(),
                message: format!("Delete book failed with error: {}", err),
                book: None,
            };

            return Ok(Response::new(reply));
        }

        let deleted_book = deleted_book.unwrap();

        if deleted_book.len() == 0 {
            let reply = DeleteBookResponse {
                status: Status::Error.into(),
                message: format!("Book with id {} does not exists", request.get_ref().id),
                book: None,
            };

            return Ok(Response::new(reply));
        }

        let rows_affected = sql_query("DELETE FROM books WHERE id = ?")
            .bind::<Integer, _>(&request.get_ref().id)
            .execute(conn);

        if let Err(err) = rows_affected {
            let reply = DeleteBookResponse {
                status: Status::Error.into(),
                message: format!("Delete book failed with error: {}", err),
                book: None,
            };

            return Ok(Response::new(reply));
        }

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

        let updated_book = sql_query("SELECT * FROM books WHERE id = ?")
            .bind::<Integer, _>(&request.get_ref().id)
            .load::<Book>(conn);

        if let Err(err) = updated_book {
            let reply = UpdateBookResponse {
                status: Status::Error.into(),
                message: format!("Update book failed with error: {}", err),
                book: None,
            };

            return Ok(Response::new(reply));
        }

        let updated_book = updated_book.unwrap();

        if updated_book.len() == 0 {
            let reply = UpdateBookResponse {
                status: Status::Error.into(),
                message: format!("Book with id {} does not exists", request.get_ref().id),
                book: None,
            };

            return Ok(Response::new(reply));
        }

        let rows_affected = sql_query("UPDATE books SET name = ? WHERE id = ?")
            .bind::<Text, _>(&request.get_ref().name)
            .bind::<Integer, _>(&request.get_ref().id)
            .execute(conn);

        let rows_affected = rows_affected.unwrap();
        if rows_affected == 0 {
            let reply = UpdateBookResponse {
                status: Status::Error.into(),
                message: format!("Updated book failed"),
                book: None,
            };

            return Ok(Response::new(reply));
        }

        let reply = UpdateBookResponse {
            status: Status::Ok.into(),
            message: format!("Updated book with id: {}", updated_book[0].id),
            book: Some(payments::Book {
                id: updated_book[0].id,
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
