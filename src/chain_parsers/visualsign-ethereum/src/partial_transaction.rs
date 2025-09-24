// Partial Transaction Decoder for Ethereum
// This module provides functionality to decode partial transactions where some fields may be missing

use alloy_primitives::{Address, Bytes, U256, hex};
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use visualsign::{
    SignablePayload, SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldTextV2,
    vsptrait::VisualSignOptions,
};

/// A partial Ethereum transaction that follows the RLP structure from the fixture
/// RLP structure: [chain_id, nonce, gas_price, gas_tip, gas_limit, to, value, data, access_list]
#[derive(Debug, Clone, PartialEq, RlpEncodable, RlpDecodable)]
pub struct PartialEthereumTransaction {
    pub chain_id: U256,       // 0x11 (17)
    pub nonce: U256,          // 0x (empty = 0)
    pub gas_price: U256,      // 0x03 (3)
    pub gas_tip: U256,        // 0x14 (20)
    pub gas_limit: U256,      // 0x5208 (21000)
    pub to: Address,          // 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    pub value: U256,          // 0x01 (1 wei, but represents 1 ETH in context)
    pub data: Bytes,          // 0x (empty)
    pub access_list: Vec<u8>, // [] (empty list)
}

impl Default for PartialEthereumTransaction {
    fn default() -> Self {
        Self {
            chain_id: U256::from(1u64), // Default to Ethereum mainnet
            nonce: U256::ZERO,
            gas_price: U256::ZERO,
            gas_tip: U256::ZERO,
            gas_limit: U256::from(21000u64), // Standard transfer gas limit
            to: Address::ZERO,
            value: U256::ZERO,
            data: Bytes::new(),
            access_list: Vec::new(),
        }
    }
}

impl PartialEthereumTransaction {
    /// Create a new partial transaction with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from the fixture data
    pub fn from_fixture() -> Self {
        Self {
            chain_id: U256::from(17u64),     // 0x11
            nonce: U256::ZERO,               // 0x (empty)
            gas_price: U256::from(3u64),     // 0x03
            gas_tip: U256::from(20u64),      // 0x14
            gas_limit: U256::from(21000u64), // 0x5208
            to: Address::from_slice(
                &hex::decode("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(),
            ),
            value: U256::from(1u64), // 0x01 (represents 1 ETH in context)
            data: Bytes::new(),      // 0x (empty)
            access_list: Vec::new(), // [] (empty)
        }
    }

    /// Decode from RLP bytes using alloy-rlp's automatic decoding
    pub fn decode_partial(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if buf.is_empty() {
            return Err("Cannot decode transaction from empty data".into());
        }

        // Use alloy-rlp's automatic decoding with the RlpDecodable derive
        let mut buf_slice = buf;
        match Self::decode(&mut buf_slice) {
            Ok(tx) => Ok(tx),
            Err(e) => Err(format!("Failed to decode RLP: {}", e).into()),
        }
    }

    /// Encode to RLP bytes using alloy-rlp's automatic encoding
    pub fn encode_partial(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.encode(&mut buffer);
        buffer
    }
    /// Convert to visual sign format
    pub fn to_visual_sign_payload(&self, options: VisualSignOptions) -> SignablePayload {
        let mut fields = Vec::new();

        // Network field
        let chain_id_u64: u64 = self.chain_id.to();
        let chain_name = match chain_id_u64 {
            1 => "Ethereum Mainnet".to_string(),
            11155111 => "Sepolia Testnet".to_string(),
            5 => "Goerli Testnet".to_string(),
            17 => "Custom Chain ID 17".to_string(), // From our fixture
            137 => "Polygon Mainnet".to_string(),
            _ => format!("Chain ID: {}", chain_id_u64),
        };

        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: chain_name.clone(),
                label: "Network".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text: chain_name },
        });

        // Transaction type field
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: "Partial Transaction".to_string(),
                label: "Transaction Type".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: "Partial Transaction".to_string(),
            },
        });

        // To address
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: self.to.to_string(),
                label: "To Address".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: self.to.to_string(),
            },
        });

        // Value - use alloy's format_units to properly format the value
        let value_text = format_value_with_unit(self.value);

        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: value_text.clone(),
                label: "Value".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text: value_text },
        });

        // Nonce
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: self.nonce.to_string(),
                label: "Nonce".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: self.nonce.to_string(),
            },
        });

        // Gas limit
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: self.gas_limit.to_string(),
                label: "Gas Limit".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: self.gas_limit.to_string(),
            },
        });

        // Gas price
        let gas_price_text = format!("{} wei", self.gas_price);
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: gas_price_text.clone(),
                label: "Gas Price".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: gas_price_text,
            },
        });

        // Gas tip (EIP-1559)
        let gas_tip_text = format!("{} wei", self.gas_tip);
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: gas_tip_text.clone(),
                label: "Gas Tip".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text: gas_tip_text },
        });

        // Input data
        if !self.data.is_empty() {
            fields.push(SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("0x{}", hex::encode(&self.data)),
                    label: "Input Data".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("0x{}", hex::encode(&self.data)),
                },
            });
        }

        let title = options
            .transaction_name
            .unwrap_or_else(|| "Partial Ethereum Transaction".to_string());

        SignablePayload::new(
            0,
            title,
            Some("Partial transaction decoded using alloy-rlp".to_string()),
            fields,
            "EthereumTx".to_string(),
        )
    }
}

// Helper function to format value with appropriate unit using alloy's format_units
fn format_value_with_unit(wei: U256) -> String {
    use alloy_primitives::utils::format_units;

    // For very small values (< 1000 wei), show as wei
    if wei < U256::from(1000u64) {
        format!("{} wei", wei)
    } else {
        // For larger values, show as ETH using alloy's format_units
        let formatted = format_units(wei, 18).unwrap_or_else(|_| wei.to_string());

        // Trim trailing zeros
        let trimmed = if formatted.contains('.') {
            formatted
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            formatted
        };

        format!("{} ETH", trimmed)
    }
}

/// Decode partial transaction from hex string using Alloy's robust parsing
pub fn decode_partial_transaction_from_hex(
    hex_data: &str,
) -> Result<PartialEthereumTransaction, Box<dyn std::error::Error>> {
    let clean_hex = hex_data.strip_prefix("0x").unwrap_or(hex_data);

    // Handle empty hex by erroring instead of making up values
    if clean_hex.is_empty() {
        return Err("Cannot decode transaction from empty hex string".into());
    }

    let bytes = hex::decode(clean_hex)?;
    PartialEthereumTransaction::decode_partial(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_transaction() {
        let fixture_tx = PartialEthereumTransaction::from_fixture();

        assert_eq!(fixture_tx.chain_id, U256::from(17u64));
        assert_eq!(fixture_tx.nonce, U256::ZERO);
        assert_eq!(fixture_tx.gas_price, U256::from(3u64));
        assert_eq!(fixture_tx.gas_tip, U256::from(20u64));
        assert_eq!(fixture_tx.gas_limit, U256::from(21000u64));
        assert_eq!(
            fixture_tx.to,
            Address::from_slice(
                &hex::decode("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap()
            )
        );
        assert_eq!(fixture_tx.value, U256::from(1u64)); // Represents 1 ETH in context
        assert_eq!(fixture_tx.data, Bytes::new());
        assert_eq!(fixture_tx.access_list, Vec::<u8>::new());
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original_tx = PartialEthereumTransaction::from_fixture();

        // Encode to RLP
        let encoded = original_tx.encode_partial();

        // Decode back from RLP
        let decoded_tx = PartialEthereumTransaction::decode_partial(&encoded).unwrap();

        // Should be identical
        assert_eq!(original_tx, decoded_tx);
    }

    #[test]
    fn test_decode_from_hex_fixture() {
        let hex_data = "df1180031482520894f39Fd6e51aad88F6F4ce6aB8827279cffFb922660180c0";

        let decoded_tx = decode_partial_transaction_from_hex(hex_data).unwrap();
        let fixture_tx = PartialEthereumTransaction::from_fixture();

        assert_eq!(decoded_tx, fixture_tx);
    }

    #[test]
    fn test_visual_sign_payload() {
        let fixture_tx = PartialEthereumTransaction::from_fixture();

        let options = VisualSignOptions {
            decode_transfers: true,
            transaction_name: Some("Test Fixture Transaction".to_string()),
            partial_parsing: true,
        };

        let payload = fixture_tx.to_visual_sign_payload(options);
        assert_eq!(payload.title, "Test Fixture Transaction");

        // Helper function to find field by label
        let find_field_text = |label: &str| -> Option<String> {
            payload.fields.iter().find_map(|f| match f {
                SignablePayloadField::TextV2 { common, text_v2 } if common.label == label => {
                    Some(text_v2.text.clone())
                }
                SignablePayloadField::Text { common, text } if common.label == label => {
                    Some(text.text.clone())
                }
                _ => None,
            })
        };

        // Check the fields match our fixture
        assert_eq!(
            find_field_text("Network"),
            Some("Custom Chain ID 17".to_string())
        );
        // Address comparison should be case-insensitive
        let to_address = find_field_text("To Address").unwrap();
        assert_eq!(
            to_address.to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
        assert_eq!(find_field_text("Value"), Some("1 wei".to_string())); // Using alloy format_units
        assert_eq!(find_field_text("Nonce"), Some("0".to_string()));
        assert_eq!(find_field_text("Gas Limit"), Some("21000".to_string()));
    }

    #[test]
    fn test_alloy_rlp_pattern() {
        // Test the pattern described in the user request
        let my_tx = PartialEthereumTransaction {
            chain_id: U256::from(17u64),
            nonce: U256::ZERO,
            gas_price: U256::from(3u64),
            gas_tip: U256::from(20u64),
            gas_limit: U256::from(21000u64),
            to: Address::from_slice(
                &hex::decode("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(),
            ),
            value: U256::from(1u64),
            data: Bytes::new(),
            access_list: Vec::new(),
        };

        let mut buffer = Vec::<u8>::new();
        let _encoded_size = my_tx.encode(&mut buffer);
        let decoded = PartialEthereumTransaction::decode(&mut buffer.as_slice()).unwrap();
        assert_eq!(my_tx, decoded);
    }
}
