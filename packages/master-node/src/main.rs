//! A simple script to generate and verify the proof of a given program.
extern crate dotenv;

mod listener;

use crate::listener::listener;

use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    loop {
        match listener().await {
            Ok(_) => (),
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
