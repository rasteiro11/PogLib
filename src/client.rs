use payments::{pog_lib_client::PogLibClient, AddBookRequest};
pub mod payments {
    tonic::include_proto!("poglib");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PogLibClient::connect("http://[::1]:42069").await?;

    let request = tonic::Request::new(AddBookRequest {
        name: "VERY COOL BOOK".into(),
    });

    let response = client.add_book(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
