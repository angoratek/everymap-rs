use async_trait::async_trait;
use crate::types::Coordinate;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for a static map image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRequest<O> {
    pub center: Coordinate,
    pub zoom: u32,
    pub size: (u32, u32),
    pub options: O,
}

/// Simplified image response from the core trait.
#[derive(Debug, Clone)]
pub struct ImageResponse {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

impl ImageResponse {
    /// Create a new image response from raw bytes.
    pub fn new(data: Vec<u8>, content_type: Option<String>) -> Self {
        Self { data, content_type }
    }
}

#[async_trait]
pub trait MapImageProvider: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_image(&self, req: ImageRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}