use async_trait::async_trait;
use crate::error::EveryMapResult;
use serde::{Deserialize, Serialize};

/// Request for map attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeRequest<O> {
    pub options: O,
}

/// Simplified attribute response from the core trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeResponse {
    /// The attribute data as JSON
    pub data: serde_json::Value,
}

#[async_trait]
pub trait AttributeProvider: Send + Sync {
    type Options: Send + Sync;
    type Response: Send + Sync;

    async fn get_attributes(&self, req: AttributeRequest<Self::Options>) -> EveryMapResult<Self::Response>;
}