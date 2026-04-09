use async_trait::async_trait;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for a map tile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRequest<O> {
    pub z: u32,
    pub x: u32,
    pub y: u32,
    pub options: O,
}

/// Simplified tile response from the core trait.
#[derive(Debug, Clone)]
pub struct TileResponse {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

#[async_trait]
pub trait TileProvider: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_tile(&self, req: TileRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}