//! Master node is responsible for listening to the proof requests and sending them to the worker node to generate the proof.
extern crate dotenv;

mod listener;

use crate::listener::listener;

use dotenv::dotenv;
use log::info;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    info!("Starting master node server.");

    loop {
        match listener().await {
            Ok(_) => (),
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }

        // Wait for 1 second before fetching proof requests again.
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
