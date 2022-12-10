use payments::{pog_lib_client::PogLibClient, AddBookRequest};
pub mod payments {
    tonic::include_proto!("poglib");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "http://0.0.0.0:42069";
    let mut client = PogLibClient::connect(addr).await?;

    let request = tonic::Request::new(AddBookRequest {
        name: "VERY COOL BOOK".into(),
    });

    let response = client.add_book(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
