use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error};
use visualsign::{SignablePayload, vsptrait::VisualSignOptions};
use visualsign_ethereum::transaction_string_to_visual_sign;

// ============================================================================
// API Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseRequest {
    /// Raw transaction hex string (with or without 0x prefix)
    pub transaction: String,

    /// Optional chain ID (defaults to 1 for Ethereum mainnet)
    #[serde(default = "default_chain_id")]
    pub chain_id: u64,

    /// Whether to decode ERC20 transfers (default: true)
    #[serde(default = "default_true")]
    pub decode_transfers: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseResponse {
    /// Successfully parsed visual sign payload
    pub payload: SignablePayload,

    /// Transaction hash (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub supported_chains: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_features: Vec<String>,
    pub eigenlayer_methods: usize,
    pub eigenlayer_coverage: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn default_chain_id() -> u64 {
    1 // Ethereum mainnet
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Invalid transaction format: {0}")]
    InvalidTransaction(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::InvalidTransaction(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::ParsingError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            error: error_message.clone(),
            details: None,
        });

        (status, body).into_response()
    }
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
struct AppState {
    // Add any shared state here (e.g., metrics, rate limiting)
}

// ============================================================================
// API Handlers
// ============================================================================

/// Health check endpoint
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        supported_chains: vec!["ethereum".to_string()],
    })
}

/// Info endpoint with EigenLayer details
async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: "VisualSign API - EigenLayer Edition".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "REST API for parsing EigenLayer transactions with 100% method coverage".to_string(),
        supported_features: vec![
            "EigenLayer (60 methods, 100% coverage)".to_string(),
            "Condensed + Expanded views".to_string(),
            "Strategy metadata resolution".to_string(),
            "Amount formatting (ETH)".to_string(),
            "Static & dynamic annotations".to_string(),
            "Badge text (Operator, AVS, Verified)".to_string(),
            "14 LST strategies supported".to_string(),
        ],
        eigenlayer_methods: 60,
        eigenlayer_coverage: "100%".to_string(),
    })
}

/// Parse transaction endpoint
async fn parse_transaction(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ParseRequest>,
) -> Result<Json<ParseResponse>, ApiError> {
    info!("Parsing transaction: chain_id={}", request.chain_id);

    // Validate and clean hex string
    let tx_hex = request.transaction.trim();
    let tx_hex = if tx_hex.starts_with("0x") {
        &tx_hex[2..]
    } else {
        tx_hex
    };

    // Validate hex format
    if !tx_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::InvalidTransaction(
            "Transaction must be a valid hex string".to_string()
        ));
    }

    // Set parsing options
    let options = VisualSignOptions {
        decode_transfers: request.decode_transfers,
        transaction_name: None,
        metadata: None,
    };

    // Parse the transaction
    let payload = transaction_string_to_visual_sign(tx_hex, options)
        .map_err(|e| {
            error!("Failed to parse transaction: {}", e);
            ApiError::ParsingError(format!("Failed to parse transaction: {}", e))
        })?;

    Ok(Json(ParseResponse {
        payload,
        tx_hash: None, // Could compute this if needed
    }))
}

// ============================================================================
// Main Application
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
        )
        .init();

    info!("Starting VisualSign API Server");

    // Create app state
    let state = Arc::new(AppState {});

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
        .route("/parse", post(parse_transaction))
        .route("/api/v1/parse", post(parse_transaction))
        .with_state(state)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    info!("Listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    info!("VisualSign API is ready!");
    info!("  - Health check: http://{}:{}/health", "localhost", port);
    info!("  - Info: http://{}:{}/info", "localhost", port);
    info!("  - Parse endpoint: POST http://{}:{}/parse", "localhost", port);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_chain_id(), 1);
        assert_eq!(default_true(), true);
    }
}
