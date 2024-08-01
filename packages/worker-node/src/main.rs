extern crate dotenv;

mod artifact;
mod prove;
mod s3;
mod server;
mod statics;

use dotenv::dotenv;

use crate::server::start_server;

#[tokio::main]
async fn main() {
    dotenv().ok();

    loop {
        match start_server().await {
            Ok(_) => (),
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
