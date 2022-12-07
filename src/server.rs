use database::database::get_connection;
use diesel::*;
use dotenvy::dotenv;
use models::models::Book;
use models::models::NewBook;
use payments::{pog_lib_server::PogLib, AddBookRequest, AddBookResponse, Status};
use std::env;
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

fn connection() -> MysqlConnection {
    dotenvy::dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in order to run unit tests");
    MysqlConnection::establish(&database_url).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let addr = "[::1]:42069".parse()?;
    //let btc_service = PogLibService::default();

    //Server::builder()
    //    .add_service(PogLibServer::new(btc_service))
    //    .serve(addr)
    //    .await?;

    dotenv().ok();

    let url = env::var("DATABASE_URL").expect("DATABASE_URL is not present in .env file");
    let mut conn = get_connection(&url);

    use crate::schema::*;

    diesel::insert_into(books::table)
        .values(NewBook { name: "TESTE 1" })
        .execute(&mut conn)
        .expect("FUCKED SAVING");

    diesel::insert_into(books::table)
        .values(NewBook { name: "TESTE 2" })
        .execute(&mut conn)
        .expect("FUCKED SAVING");
    //  let connection = &mut connection();
    //let res: Book = books.order(id.desc()).first::<Book>(connection)?;

    let conn_2 = &mut connection();
    let query = sql_query("SELECT * FROM books").execute(conn_2);
    let bs: Vec<Book> = books::table.load(conn_2).unwrap();

    println!("LISTING ALL BOOKS");
    for book in bs {
        println!("{:?}", book);
    }

    // println!("{:?}", res);

    Ok(())
}
