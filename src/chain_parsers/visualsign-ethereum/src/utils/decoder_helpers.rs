//! Generic helper functions for decoding Solidity contract structs using sol! macro
//!
//! This module provides reusable patterns for:
//! - Decoding struct parameters using alloy's `abi_decode`
//! - Handling decode errors gracefully
//! - Creating error fields when decoding fails
//!
//! # Example
//!
//! ```rust,ignore
//! use visualsign_ethereum::utils::decoder_helpers::decode_or_error;
//!
//! // Decode a struct parameter, returning error field if it fails
//! let params = decode_or_error::<MyParams>(bytes, "Operation Name")?;
//! ```

use visualsign::{SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldTextV2};

/// Macro to create an error field when decoding fails
///
/// Returns a TextV2 field with the operation name and fallback
#[macro_export]
macro_rules! decode_error_field {
    ($op_name:expr, $bytes:expr) => {
        $crate::SignablePayloadField::TextV2 {
            common: $crate::SignablePayloadFieldCommon {
                fallback_text: format!("{}: 0x{}", $op_name, hex::encode($bytes)),
                label: $op_name.to_string(),
            },
            text_v2: $crate::SignablePayloadFieldTextV2 {
                text: "Failed to decode parameters".to_string(),
            },
        }
    };
}

/// Helper to create a generic error field for decoding failures
///
/// Used when you need more control over error formatting
pub fn error_field(operation_name: &str, bytes: &[u8], reason: &str) -> SignablePayloadField {
    SignablePayloadField::TextV2 {
        common: SignablePayloadFieldCommon {
            fallback_text: format!("{}: 0x{}", operation_name, hex::encode(bytes)),
            label: operation_name.to_string(),
        },
        text_v2: SignablePayloadFieldTextV2 {
            text: reason.to_string(),
        },
    }
}
