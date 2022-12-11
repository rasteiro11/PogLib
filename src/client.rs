use std::{
    fs::{self, File},
    io::Write,
};
extern crate base64;

use payments::{pog_lib_client::PogLibClient, AddBookRequest, GetBookRequest};
pub mod payments {
    tonic::include_proto!("poglib");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://0.0.0.0:42069";
    let mut client = PogLibClient::connect(addr).await?;

    // ADD NEW BOOK
    //let blob = fs::read("/home/titico03/Documents/RUST/PogLib/books_client/sample.png")
    //    .expect("READING FILE IS FUCKED");
    //let encoded_file = base64::encode(blob);

    //let request = tonic::Request::new(AddBookRequest {
    //    encoded_file,
    //    name: "sample.png".into(),
    //});

    //let response = client.add_book(request).await?;

    //println!("RESPONSE={:?}", response);

    // GET BOOK
    let response = client
        .get_book_by_id(tonic::Request::new(GetBookRequest { id: 48 }))
        .await?;

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

    Ok(())
}
