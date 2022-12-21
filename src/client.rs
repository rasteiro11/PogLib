use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

extern crate base64;

use futures::executor::block_on;
use payments::{pog_lib_client::PogLibClient, GetBookRequest};
use tonic::transport::Channel;

use crate::payments::{AddBookRequest, ListBooksPagesRequest, Status};
pub mod payments {
    tonic::include_proto!("poglib");
}

trait IRouter<'a, T> {
    fn add_handler(&mut self, route: char, handler: &'a dyn Fn(&mut T));
    fn exec_route(&mut self, route: char);
}

struct Router<'a, T> {
    routes: HashMap<char, &'a dyn Fn(&mut T)>,
    client: T,
}

impl<'a, T> Router<'a, T> {
    pub fn new(client: T) -> Self {
        Router {
            routes: HashMap::new(),
            client,
        }
    }
}

impl<'a, T> IRouter<'a, T> for Router<'a, T> {
    fn add_handler(&mut self, route: char, handler: &'a dyn Fn(&mut T)) {
        self.routes.insert(route, handler);
    }
    fn exec_route(&mut self, route: char) {
        let f = self.routes.get(&route);
        if let Some(f) = f {
            f(&mut self.client)
        } else {
            println!("This route was not declared");
        }
    }
}

async fn get_book(client: &mut PogLibClient<Channel>) {
    print!("Book Id: ");
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Could not get input from user");

    let index = line.trim().parse::<i32>().expect("Could not parse id");

    let response = client
        .get_book_by_id(tonic::Request::new(GetBookRequest { id: index }))
        .await
        .expect("Could not get book");

    let book = response.into_inner();
    let decoded_file = base64::decode(book.encoded_file).expect("could not decode file");

    let book_name = book.book.unwrap().name;

    let file = File::create(format!("./books_client/{}", book_name));
    match file {
        Err(err) => eprintln!("File::create() returned error: {}", err),
        Ok(mut file) => {
            file.write_all(&decoded_file)
                .expect("Could not write to file");
            println!("Everything is ok file downloaded");
        }
    }
}

async fn add_new_book(client: &mut PogLibClient<Channel>) {
    print!("Book Path: ");
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Could not get input from user");

    line = line.trim().to_string();

    let blob = fs::read(line.as_str()).expect("READING FILE IS FUCKED");
    let encoded_file = base64::encode(blob);

    let file_name = Path::new(&line)
        .file_name()
        .expect("Could not get file name")
        .to_str()
        .expect("Could not get str");

    let request = tonic::Request::new(AddBookRequest {
        encoded_file,
        name: file_name.to_string(),
    });

    let response = client
        .add_book(request)
        .await
        .expect("Book could not be added by some reason")
        .into_inner();

    let num_status = Status::from_i32(response.status).expect("Expected valid status");

    if num_status == Status::Error || num_status == Status::UnknownStatus {
        println!("Something went wrong during book download, try again later");
    } else {
        println!("Added new book");
    }
}

async fn list_books(client: &mut PogLibClient<Channel>) {
    print!("Search Book Name: ");
    //let mut line = String::new();
    //std::io::stdin()
    //    .read_line(&mut line)
    //    .expect("Could not get input from user");

    //line = line.trim().to_string();

    let request = tonic::Request::new(ListBooksPagesRequest {
        id: 0,
        name: "".to_string(),
        per_page: 100,
        page: 1,
    });

    let response = client
        .list_books(request)
        .await
        .expect("Book could not be added by some reason")
        .into_inner();

    let num_status = Status::from_i32(response.status).expect("Expected valid status");

    if num_status == Status::Error || num_status == Status::UnknownStatus {
        println!("{}", response.message);
        println!("Something went wrong during book download, try again later");
    } else {
        for book in response.data {
            println!("{:?}", book);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://0.0.0.0:42069";
    let client = PogLibClient::connect(addr).await?;

    let mut router = Router::new(client);
    router.add_handler('g', &|client: &mut PogLibClient<Channel>| {
        block_on(get_book(client));
    });
    router.add_handler('a', &|client: &mut PogLibClient<Channel>| {
        block_on(add_new_book(client));
    });
    router.add_handler('l', &|client: &mut PogLibClient<Channel>| {
        block_on(list_books(client));
    });

    while true {
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .expect("Could not get input from user");

        let route = line.bytes().nth(0).expect("Something went wrong") as char;

        router.exec_route(route);
    }

    Ok(())
}
