use visualsign::{
    SignablePayload, SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldTextV2,
    encodings::SupportedEncodings,
    vsptrait::{
        Transaction, TransactionParseError, VisualSignConverter, VisualSignConverterFromString,
        VisualSignError, VisualSignOptions,
    },
};

use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
use pallas_codec::minicbor;
use pallas_crypto::hash::Hasher;
use pallas_primitives::babbage::{MintedTx, TransactionInput, Value};

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum CardanoParserError {
    #[error("Failed to decode transaction: {0}")]
    FailedToDecodeTransaction(String),
}

fn decode_transaction<'a>(
    raw_transaction: &str,
    encodings: SupportedEncodings,
) -> Result<MintedTx<'a>, CardanoParserError> {
    let bytes = match encodings {
        SupportedEncodings::Hex => {
            let clean_hex = raw_transaction
                .strip_prefix("0x")
                .unwrap_or(raw_transaction);
            hex::decode(clean_hex).map_err(|e| {
                CardanoParserError::FailedToDecodeTransaction(format!(
                    "Failed to decode hex: {}",
                    e
                ))
            })?
        }
        SupportedEncodings::Base64 => b64.decode(raw_transaction).map_err(|e| {
            CardanoParserError::FailedToDecodeTransaction(format!("Failed to decode base64: {}", e))
        })?,
    };

    // Parse and return the Cardano transaction using minicbor
    // Note: We need to leak the bytes to get a 'static lifetime for MintedTx
    // This is a known limitation when working with borrowed CBOR data
    let bytes_static = Box::leak(bytes.into_boxed_slice());
    minicbor::decode(bytes_static).map_err(|e| {
        CardanoParserError::FailedToDecodeTransaction(format!(
            "Failed to parse Cardano transaction: {}",
            e
        ))
    })
}

/// Wrapper for Cardano transactions
#[derive(Debug, Clone)]
pub struct CardanoTransactionWrapper<'a> {
    transaction: MintedTx<'a>,
}

impl<'a> Transaction for CardanoTransactionWrapper<'a> {
    fn from_string(data: &str) -> Result<Self, TransactionParseError> {
        let format = if data.starts_with("0x") {
            SupportedEncodings::Hex
        } else {
            visualsign::encodings::SupportedEncodings::detect(data)
        };
        let transaction = decode_transaction(data, format)
            .map_err(|e| TransactionParseError::DecodeError(e.to_string()))?;
        Ok(Self { transaction })
    }

    fn transaction_type(&self) -> String {
        "Cardano".to_string()
    }
}

impl<'a> CardanoTransactionWrapper<'a> {
    pub fn new(transaction: MintedTx<'a>) -> Self {
        Self { transaction }
    }

    pub fn inner(&self) -> &MintedTx<'a> {
        &self.transaction
    }
}

/// Converter for Cardano transactions
pub struct CardanoVisualSignConverter;

impl<'a> VisualSignConverter<CardanoTransactionWrapper<'a>> for CardanoVisualSignConverter {
    fn to_visual_sign_payload(
        &self,
        transaction_wrapper: CardanoTransactionWrapper<'a>,
        options: VisualSignOptions,
    ) -> Result<SignablePayload, VisualSignError> {
        convert_to_visual_sign_payload(transaction_wrapper.inner().clone(), options)
    }
}

fn convert_to_visual_sign_payload<'a>(
    tx: MintedTx<'a>,
    options: VisualSignOptions,
) -> Result<SignablePayload, VisualSignError> {
    let chain_name = "Cardano".to_string();

    let mut fields = vec![SignablePayloadField::TextV2 {
        common: SignablePayloadFieldCommon {
            fallback_text: "Cardano".to_string(),
            label: "Network".to_string(),
        },
        text_v2: SignablePayloadFieldTextV2 { text: chain_name },
    }];

    // Calculate transaction hash
    let tx_body_bytes = minicbor::to_vec(&tx.transaction_body).map_err(|e| {
        VisualSignError::ParseError(TransactionParseError::DecodeError(format!(
            "Failed to serialize transaction body: {}",
            e
        )))
    })?;

    let mut hasher = Hasher::<256>::new();
    hasher.input(&tx_body_bytes);
    let tx_hash = hasher.finalize();
    let tx_hash_hex = hex::encode(tx_hash);

    // Add Transaction ID field
    fields.push(SignablePayloadField::TextV2 {
        common: SignablePayloadFieldCommon {
            fallback_text: tx_hash_hex.clone(),
            label: "Transaction ID".to_string(),
        },
        text_v2: SignablePayloadFieldTextV2 { text: tx_hash_hex },
    });

    // Add fee field
    let fee = tx.transaction_body.fee;
    let fee_ada = fee as f64 / 1_000_000.0;
    fields.push(SignablePayloadField::TextV2 {
        common: SignablePayloadFieldCommon {
            fallback_text: format!("{} lovelace ({:.6} ADA)", fee, fee_ada),
            label: "Fee".to_string(),
        },
        text_v2: SignablePayloadFieldTextV2 {
            text: format!("{} lovelace ({:.6} ADA)", fee, fee_ada),
        },
    });

    // Add TTL (time to live) field if present
    if let Some(ttl) = tx.transaction_body.ttl {
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Slot {}", ttl),
                label: "TTL (Time To Live)".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: format!("Slot {}", ttl),
            },
        });
    }

    // Add validity interval start if present
    if let Some(validity_start) = tx.transaction_body.validity_interval_start {
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("Slot {}", validity_start),
                label: "Validity Interval Start".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: format!("Slot {}", validity_start),
            },
        });
    }

    // Parse inputs
    for (idx, input) in tx.transaction_body.inputs.iter().enumerate() {
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: format_transaction_input(input),
                label: format!("Input {}", idx + 1),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: format_transaction_input(input),
            },
        });
    }

    // Parse outputs
    // Note: Outputs contain address and value information, but the structure
    // is complex with nested types. For now, we'll serialize each output to CBOR
    // and display a summary
    fields.push(SignablePayloadField::TextV2 {
        common: SignablePayloadFieldCommon {
            fallback_text: format!("{} output(s)", tx.transaction_body.outputs.len()),
            label: "Outputs".to_string(),
        },
        text_v2: SignablePayloadFieldTextV2 {
            text: format!("{} output(s)", tx.transaction_body.outputs.len()),
        },
    });

    // Add minting information if present
    if let Some(mint) = &tx.transaction_body.mint {
        if !mint.is_empty() {
            fields.push(SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("{} asset(s) minted/burned", mint.len()),
                    label: "Minting".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("{} asset(s) minted/burned", mint.len()),
                },
            });
        }
    }

    // Add metadata hash if present
    if let Some(metadata_hash) = &tx.transaction_body.auxiliary_data_hash {
        let hash_bytes: &[u8] = metadata_hash.as_ref();
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: hex::encode(hash_bytes),
                label: "Metadata Hash".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: hex::encode(hash_bytes),
            },
        });
    }

    // Add collateral inputs if present
    if let Some(collateral) = &tx.transaction_body.collateral {
        if !collateral.is_empty() {
            for (idx, input) in collateral.iter().enumerate() {
                fields.push(SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format_transaction_input(input),
                        label: format!("Collateral {}", idx + 1),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: format_transaction_input(input),
                    },
                });
            }
        }
    }

    // Add required signers if present
    if let Some(required_signers) = &tx.transaction_body.required_signers {
        if !required_signers.is_empty() {
            for (idx, signer) in required_signers.iter().enumerate() {
                fields.push(SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: hex::encode(signer.as_ref()),
                        label: format!("Required Signer {}", idx + 1),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: hex::encode(signer.as_ref()),
                    },
                });
            }
        }
    }

    // Add network ID if present
    if let Some(network_id) = tx.transaction_body.network_id {
        let network_name = match network_id {
            pallas_primitives::alonzo::NetworkId::One => "Mainnet",
            pallas_primitives::alonzo::NetworkId::Two => "Testnet",
        };
        fields.push(SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: format!("{} ({:?})", network_name, network_id),
                label: "Network ID".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 {
                text: format!("{} ({:?})", network_name, network_id),
            },
        });
    }

    let title = options
        .transaction_name
        .unwrap_or_else(|| "Cardano Transaction".to_string());

    Ok(SignablePayload::new(
        0,
        title,
        None,
        fields,
        "CardanoTx".to_string(),
    ))
}

impl<'a> VisualSignConverterFromString<CardanoTransactionWrapper<'a>>
    for CardanoVisualSignConverter
{
}

// Public API functions
pub fn transaction_to_visual_sign<'a>(
    transaction: MintedTx<'a>,
    options: VisualSignOptions,
) -> Result<SignablePayload, VisualSignError> {
    let wrapper = CardanoTransactionWrapper::new(transaction);
    let converter = CardanoVisualSignConverter;
    converter.to_visual_sign_payload(wrapper, options)
}

pub fn transaction_string_to_visual_sign(
    transaction_data: &str,
    options: VisualSignOptions,
) -> Result<SignablePayload, VisualSignError> {
    let converter = CardanoVisualSignConverter;
    converter.to_visual_sign_payload_from_string(transaction_data, options)
}

// Helper function to format transaction input
fn format_transaction_input(input: &TransactionInput) -> String {
    format!(
        "{}#{}",
        hex::encode(input.transaction_id.as_ref()),
        input.index
    )
}

// Helper function to format value (ADA + multi-assets)
// Currently unused but kept for future enhancements when we properly parse outputs
#[allow(dead_code)]
fn format_value(value: &Value) -> String {
    match value {
        Value::Coin(coin) => {
            let ada = *coin as f64 / 1_000_000.0;
            format!("{} lovelace ({:.6} ADA)", coin, ada)
        }
        Value::Multiasset(coin, assets) => {
            let ada = *coin as f64 / 1_000_000.0;
            let mut result = format!("{} lovelace ({:.6} ADA)", coin, ada);

            if !assets.is_empty() {
                result.push_str(&format!(" + {} asset type(s)", assets.len()));
            }

            result
        }
    }
}
