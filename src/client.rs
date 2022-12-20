use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
};

extern crate base64;

use futures::executor::block_on;
use payments::{pog_lib_client::PogLibClient, GetBookRequest};
use tonic::transport::Channel;

use crate::payments::AddBookRequest;
pub mod payments {
    tonic::include_proto!("poglib");
}

trait IRouter<'a> {
    fn add_handler(&mut self, route: char, handler: &'a dyn Fn(&mut PogLibClient<Channel>));
    fn exec_route(&mut self, route: char);
}

struct Router<'a> {
    routes: HashMap<char, &'a dyn Fn(&mut PogLibClient<Channel>)>,
    client: PogLibClient<Channel>,
}

impl<'a> Router<'a> {
    pub fn new(client: PogLibClient<Channel>) -> Self {
        Router {
            routes: HashMap::new(),
            client,
        }
    }
}

impl<'a> IRouter<'a> for Router<'a> {
    fn add_handler(&mut self, route: char, handler: &'a dyn Fn(&mut PogLibClient<Channel>)) {
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
    println!("Book Id: ");
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
        Ok(mut file) => file
            .write_all(&decoded_file)
            .expect("Could not write to file"),
    }
}

async fn add_new_book(client: &mut PogLibClient<Channel>) {
    println!("Book Path: ");
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Could not get input from user");

    let blob = fs::read(line.trim()).expect("READING FILE IS FUCKED");
    let encoded_file = base64::encode(blob);

    let request = tonic::Request::new(AddBookRequest {
        encoded_file,
        name: line.into(),
    });

    let response = client
        .add_book(request)
        .await
        .expect("Book could not be added by some reason");

    println!("RESPONSE={:?}", response);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://0.0.0.0:42069";
    let mut client = PogLibClient::connect(addr).await?;
    let mut client_2 = PogLibClient::connect(addr).await?;

    let mut router = Router::new(client_2);
    router.add_handler('l', &|client: &mut PogLibClient<Channel>| {
        block_on(get_book(client));
    });
    router.add_handler('a', &|client: &mut PogLibClient<Channel>| {
        block_on(add_new_book(client));
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
