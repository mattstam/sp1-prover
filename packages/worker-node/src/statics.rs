use aws_sdk_s3::Client as S3Client;
use lazy_static::lazy_static;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use std::{
    env,
    sync::{Arc, Mutex},
};
use tokio::sync::{OnceCell, Semaphore};

lazy_static! {
    pub static ref S3_CLIENT: OnceCell<S3Client> = OnceCell::new();
    pub static ref S3_CONCURRENCY: usize = env::var("S3_CONCURRENCY")
        .unwrap_or("32".to_string())
        .parse()
        .unwrap();
    pub static ref SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(*S3_CONCURRENCY));
    pub static ref S3_BUCKET: String = env::var("S3_BUCKET").expect("S3_BUCKET is not set");
    pub static ref HTTP_CLIENT_WITH_MIDDLEWARE: Mutex<ClientWithMiddleware> = Mutex::new({
        let reqwest_client = Client::new();

        // Create a retry policy
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        // Create the middleware with the retry policy
        let retry_middleware = RetryTransientMiddleware::new_with_policy(retry_policy);

        // Wrap the reqwest client with the middleware to create a `ClientWithMiddleware`
        ClientBuilder::new(reqwest_client)
            .with(retry_middleware)
            .build()
    });
}
