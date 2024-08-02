//! S3 operations for artifacts.

use anyhow::Result;
use aws_config::{retry::RetryConfig, BehaviorVersion};
use aws_sdk_s3::{
    config::StalledStreamProtectionConfig,
    primitives::{ByteStream, SdkBody},
    Client as S3Client,
};
use bytes::Bytes;
use futures::future::join_all;
use reqwest_middleware::ClientWithMiddleware as HttpClientWithMiddleware;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug_span, instrument};

use crate::{
    artifact::Artifact,
    statics::{S3_BUCKET, S3_CLIENT, S3_CONCURRENCY, SEMAPHORE},
};

const CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Get an S3 client.
async fn get_s3_client() -> &'static S3Client {
    S3_CLIENT
        .get_or_init(|| async {
            let mut base = aws_config::load_defaults(BehaviorVersion::latest())
                .await
                .to_builder();
            base.set_retry_config(Some(RetryConfig::standard()));
            base = base.stalled_stream_protection(StalledStreamProtectionConfig::disabled());
            let config = base.build();
            S3Client::new(&config)
        })
        .await
}

/// Download a file from S3 using parallelization.
async fn par_download_file<T: DeserializeOwned>(client: &S3Client, id: &str) -> Result<T> {
    let key = format!("artifacts/{}", id);
    let size = client
        .head_object()
        .bucket((*S3_BUCKET).clone())
        .key(key.clone())
        .send()
        .await?
        .content_length
        .unwrap();
    let mut buf = vec![0_u8; size as usize];
    let starts = (0..size).step_by(CHUNK_SIZE);
    let chunks = buf.chunks_mut(CHUNK_SIZE);
    let threads = std::cmp::min(*S3_CONCURRENCY, chunks.len());
    // Split into up to S3_CONCURRENCY threads. For each thread, acquire a permit and download chunks.
    let mut chunk_inputs = starts.into_iter().zip(chunks).collect::<Vec<_>>();
    let futures = chunk_inputs.chunks_mut(threads).map(|chunk_inputs| {
        let client = client.clone();
        let key = key.clone();
        async move {
            let _permit = SEMAPHORE.acquire().await.unwrap();
            for (start, chunk) in chunk_inputs {
                let end = std::cmp::min(*start + chunk.len() as i64, size) - 1;
                let res = client
                    .get_object()
                    .bucket((*S3_BUCKET).clone())
                    .key(key.clone())
                    .range(format!("bytes={}-{}", start, end))
                    .send()
                    .await?;
                let body = res.body.collect().await.unwrap();
                chunk.copy_from_slice(&body.to_vec());
            }
            Ok::<(), anyhow::Error>(())
        }
    });
    join_all(futures).await;

    let deserialized = debug_span!("deserialize").in_scope(|| bincode::deserialize(&buf))?;
    Ok(deserialized)
}

/// Upload a file to S3 using parallelization.
async fn par_upload_file<T: Serialize>(client: &S3Client, id: &str, item: T) -> Result<()> {
    let data = debug_span!("serialize").in_scope(|| bincode::serialize(&item))?;
    let key = format!("artifacts/{}", id);
    let create_multipart_upload = client
        .create_multipart_upload()
        .bucket((*S3_BUCKET).clone())
        .key(key.clone())
        .send()
        .await?;

    let upload_id = create_multipart_upload.upload_id().unwrap();

    // Upload in parallel
    let threads = std::cmp::min(*S3_CONCURRENCY, data.len());
    // Split into up to S3_CONCURRENCY threads. For each thread, acquire a permit and upload chunks.
    let num_chunks = std::cmp::max((data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE, 1);
    let mut parts = vec![None; num_chunks];
    let mut chunk_inputs = data
        .chunks(CHUNK_SIZE)
        .zip(parts.iter_mut())
        .enumerate()
        .collect::<Vec<_>>();
    let futures = chunk_inputs.chunks_mut(threads).map(|chunk_inputs| {
        let client = client.clone();
        let key = key.clone();
        async move {
            let _permit = SEMAPHORE.acquire().await.unwrap();
            for (i, (chunk, part_option)) in chunk_inputs {
                let bytes = Bytes::from(chunk.to_vec());
                let body = ByteStream::new(SdkBody::from(bytes));
                let upload_part = client
                    .upload_part()
                    .bucket((*S3_BUCKET).clone())
                    .key(key.clone())
                    .upload_id(upload_id)
                    .body(body)
                    .part_number(*i as i32 + 1)
                    .send();
                let part = upload_part.await.unwrap();

                part_option.replace(
                    aws_sdk_s3::types::CompletedPart::builder()
                        .e_tag(part.e_tag().unwrap())
                        .part_number(*i as i32 + 1)
                        .build(),
                );
            }
        }
    });
    join_all(futures).await;

    let upload_parts = parts
        .into_iter()
        .map(|part_option| part_option.unwrap())
        .collect::<Vec<_>>();

    client
        .complete_multipart_upload()
        .bucket((*S3_BUCKET).clone())
        .key(key.clone())
        .upload_id(upload_id)
        .multipart_upload(
            aws_sdk_s3::types::CompletedMultipartUpload::builder()
                .set_parts(Some(upload_parts))
                .build(),
        )
        .send()
        .await?;

    Ok(())
}

impl Artifact {
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            expiry: None,
        }
    }

    #[instrument(name = "download", level = "info", fields(label = self.label, id = self.id), skip_all)]
    pub async fn download<T: DeserializeOwned>(
        &self,
        _client: &HttpClientWithMiddleware,
    ) -> Result<T> {
        let s3_client = get_s3_client().await;
        par_download_file(s3_client, &self.id).await
    }

    #[instrument(name = "upload", level = "info", fields(label = self.label, id = self.id), skip_all)]
    pub async fn upload<T: Serialize>(
        &self,
        _client: &HttpClientWithMiddleware,
        item: T,
    ) -> Result<()> {
        let s3_client = get_s3_client().await;
        par_upload_file(s3_client, &self.id, item).await
    }
}
