//! Ethereum-specific field builder extensions
//!
//! These functions build on top of the `visualsign` field builders to provide
//! Ethereum-specific functionality:
//! - Token amount formatting using registry and decimals
//! - Address fields with token names and optional badges
//! - Percentage fields (for Uniswap bips, Aave percentages, etc.)
//! - Swap information fields
//!
//! # Design Philosophy
//!
//! These builders combine registry lookups with field builder helpers to provide
//! a clean developer experience. Any protocol can use these to visualize transfers,
//! swaps, approvals, and other common operations.
//!
//! # Example
//!
//! ```rust,ignore
//! use visualsign_ethereum::utils::field_builders_ext::{
//!     create_token_amount_field, create_token_address_field
//! };
//!
//! // Create a field showing token amount with symbol
//! let amount_field = create_token_amount_field(
//!     "Amount In",
//!     1_500_000_000u128,  // Raw amount (with decimals)
//!     chain_id,
//!     token_address,
//!     registry,
//! )?;
//!
//! // Create a field for a token address with badge
//! let token_field = create_token_address_field(
//!     "Token",
//!     token_address,
//!     chain_id,
//!     registry,
//!     Some("Input"),
//! )?;
//! ```

use alloy_primitives::Address;
use visualsign::field_builders::*;
use visualsign::AnnotatedPayloadField;
use crate::registry::ContractRegistry;

/// Create an amount field for a token using registry metadata
///
/// This function:
/// 1. Looks up the token in the registry for decimals
/// 2. Formats the amount with proper decimal places and symbol
/// 3. Returns a field ready for visualization
///
/// # Arguments
/// * `label` - Field label (e.g., "Amount In")
/// * `raw_amount` - The amount in raw form (needs decimals applied)
/// * `chain_id` - Chain ID for registry lookup
/// * `token_address` - Token contract address
/// * `registry` - Optional registry for token metadata
///
/// # Returns
/// If registry has the token metadata, returns formatted amount with symbol.
/// Otherwise, returns the raw amount as a string.
pub fn create_token_amount_field(
    label: &str,
    raw_amount: u128,
    chain_id: u64,
    token_address: Address,
    registry: Option<&ContractRegistry>,
) -> Result<AnnotatedPayloadField, Box<dyn std::error::Error>> {
    // Try to get formatted amount from registry
    let (amount_str, symbol) = registry
        .and_then(|r| r.format_token_amount(chain_id, token_address, raw_amount))
        .unwrap_or_else(|| {
            // Fallback: raw amount without symbol
            (raw_amount.to_string(), "tokens".to_string())
        });

    create_amount_field(label, &amount_str, &symbol)
}

/// Create an address field for a token with optional badge
///
/// This function:
/// 1. Looks up token symbol in the registry
/// 2. Creates an address field with the contract name if available
/// 3. Optionally adds a badge (e.g., "Input", "Output", "Fee")
///
/// # Arguments
/// * `label` - Field label (e.g., "Token")
/// * `token_address` - The token address
/// * `chain_id` - Chain ID for registry lookup
/// * `registry` - Optional registry for metadata
/// * `badge` - Optional badge text (e.g., "Input", "Fee")
pub fn create_token_address_field(
    label: &str,
    token_address: Address,
    chain_id: u64,
    registry: Option<&ContractRegistry>,
    badge: Option<&str>,
) -> Result<AnnotatedPayloadField, Box<dyn std::error::Error>> {
    // Try to get symbol from registry
    let token_name = registry
        .and_then(|r| r.get_token_symbol(chain_id, token_address));

    create_address_field(
        label,
        &format!("{:?}", token_address),
        token_name.as_deref(),
        None,
        None,
        badge,
    )
}

/// Create a percentage field from basis points (bips)
///
/// Converts bips to human-readable percentage:
/// - 10000 bips = 100%
/// - 100 bips = 1%
/// - 1 bips = 0.01%
///
/// # Arguments
/// * `label` - Field label (e.g., "Fee", "Slippage", "Commission")
/// * `bips` - Basis points value
pub fn create_bips_field(label: &str, bips: u32) -> Result<AnnotatedPayloadField, Box<dyn std::error::Error>> {
    let percent = (bips as f64) / 100.0;
    let percentage_str = if percent >= 1.0 {
        format!("{:.2}%", percent)
    } else {
        format!("{:.4}%", percent)
    };

    create_text_field(label, &percentage_str)
}

/// Create a recipient/destination address field
///
/// Commonly used in swaps, transfers, and other operations
pub fn create_recipient_field(
    recipient: Address,
    label: Option<&str>,
) -> Result<AnnotatedPayloadField, Box<dyn std::error::Error>> {
    create_address_field(
        label.unwrap_or("Recipient"),
        &format!("{:?}", recipient),
        None,
        None,
        None,
        None,
    )
}

/// Create a swap information summary
///
/// Creates a text field summarizing the swap direction and tokens
///
/// # Arguments
/// * `input_token` - Input token symbol
/// * `output_token` - Output token symbol
/// * `input_amount` - Formatted input amount
/// * `output_amount` - Formatted minimum output amount
pub fn create_swap_summary_field(
    input_token: &str,
    output_token: &str,
    input_amount: &str,
    output_amount: &str,
) -> Result<AnnotatedPayloadField, Box<dyn std::error::Error>> {
    let summary = format!(
        "Swap {} {} for ≥{} {}",
        input_amount, input_token, output_amount, output_token
    );
    create_text_field("Swap", &summary)
}
