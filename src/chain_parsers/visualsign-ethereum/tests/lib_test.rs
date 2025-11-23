use std::fs;
use std::path::PathBuf;
use visualsign::vsptrait::VisualSignOptions;
use visualsign::SignablePayloadField;
use visualsign_ethereum::{transaction_string_to_visual_sign, transaction_to_visual_sign};
use alloy_consensus::{TxLegacy, TypedTransaction};
use alloy_primitives::{Address, Bytes, ChainId, U256};

// Helper function to get fixture path
fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path
}

static FIXTURES: [&str; 2] = ["1559", "legacy"];

#[test]
fn test_with_fixtures() {
    // Get paths for all test cases
    let fixtures_dir = fixture_path("");

    for test_name in FIXTURES {
        let input_path = fixtures_dir.join(format!("{test_name}.input"));

        // Read input file contents
        let input_contents = fs::read_to_string(&input_path)
            .unwrap_or_else(|_| panic!("Failed to read input file: {input_path:?}"));

        // Parse the input to extract transaction data
        let transaction_hex = input_contents.trim();

        // Create options for the transaction
        let options = VisualSignOptions {
            decode_transfers: true,
            transaction_name: None,
            metadata: None,
        };

        let result = transaction_string_to_visual_sign(transaction_hex, options);

        let actual_output = match result {
            Ok(payload) => payload.to_json().unwrap(),
            Err(error) => format!("Error: {error:?}"),
        };

        // Construct expected output path
        let expected_path = fixtures_dir.join(format!("{test_name}.expected"));

        // Read expected output
        let expected_output = fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("Expected output file not found: {expected_path:?}"));

        assert_eq!(
            actual_output.trim(),
            expected_output.trim(),
            "Test case '{test_name}' failed",
        );
    }
}

#[test]
fn test_ethereum_charset_validation() {
    // Test that Ethereum parser produces ASCII-only output
    let fixtures_dir = fixture_path("");

    for test_name in FIXTURES {
        let input_path = fixtures_dir.join(format!("{test_name}.input"));

        // Read input file contents
        let input_contents = fs::read_to_string(&input_path)
            .unwrap_or_else(|_| panic!("Failed to read input file: {input_path:?}"));

        // Parse the input to extract transaction data
        let transaction_hex = input_contents.trim();

        // Create options for the transaction
        let options = VisualSignOptions {
            decode_transfers: true,
            transaction_name: None,
            metadata: None,
        };

        let result = transaction_string_to_visual_sign(transaction_hex, options);

        match result {
            Ok(payload) => {
                // Test charset validation
                let validation_result = payload.validate_charset();
                assert!(
                    validation_result.is_ok(),
                    "Ethereum parser should produce ASCII-only output for test case '{}', got validation error: {:?}",
                    test_name,
                    validation_result.err()
                );

                // Test that to_validated_json works
                let json_result = payload.to_validated_json();
                assert!(
                    json_result.is_ok(),
                    "Ethereum parser output should serialize with charset validation for test case '{}', got error: {:?}",
                    test_name,
                    json_result.err()
                );

                let json_string = json_result.unwrap();

                // Verify specific unicode escapes are not present
                let unicode_escapes = vec!["\\u003e", "\\u003c", "\\u0026", "\\u0027", "\\u002b"];
                for escape in unicode_escapes {
                    assert!(
                        !json_string.contains(escape),
                        "Ethereum parser JSON should not contain unicode escape {escape} for test case '{test_name}', but found in: {}",
                        json_string.chars().take(200).collect::<String>()
                    );
                }

                // Verify the JSON is valid ASCII
                assert!(
                    json_string.is_ascii(),
                    "Ethereum parser JSON output should be ASCII only for test case '{test_name}'",
                );
            }
            Err(error) => {
                // If parsing fails, that's okay for this test - we're only testing
                // that successful parses produce valid charsets
                eprintln!(
                    "Skipping charset validation for test case '{test_name}' due to parse error: {error:?}",
                );
            }
        }
    }
}

#[test]
fn test_eigenlayer_deposit_into_strategy() {
    // Real EigenLayer depositIntoStrategy transaction
    // From: https://etherscan.io/tx/0x16edc73c42ba76077008fb5f449e36d26e70e939af3285082e00a533202ae654
    let input_data = hex::decode("e7a050aa00000000000000000000000093c4b944d05dfe6df7645a86cd2206016c51564d000000000000000000000000ae7ab96520de3a18e5e111b5eaab095312d7fe8400000000000000000000000000000000000000000000000000028b30699cdc00").unwrap();

    let tx = TypedTransaction::Legacy(TxLegacy {
        chain_id: Some(ChainId::from(1u64)),
        nonce: 12,
        gas_price: 101521211u128,
        gas_limit: 960000,
        to: alloy_primitives::TxKind::Call("0x858646372CC42E1A627fcE94aa7A7033e7CF075A".parse().unwrap()),
        value: U256::ZERO,
        input: Bytes::from(input_data),
    });

    let options = VisualSignOptions::default();
    let payload = transaction_to_visual_sign(tx, options).unwrap();

    // Check that EigenLayer deposit is detected
    let has_eigenlayer = payload.fields.iter().any(|f| {
        if let SignablePayloadField::PreviewLayout { preview_layout, .. } = f {
            if let Some(title) = &preview_layout.title {
                return title.text.contains("EigenLayer");
            }
        }
        false
    });

    assert!(has_eigenlayer, "Should detect EigenLayer deposit transaction");

    // Check for specific EigenLayer fields
    let eigenlayer_field = payload.fields.iter().find(|f| {
        if let SignablePayloadField::PreviewLayout { preview_layout, .. } = f {
            if let Some(title) = &preview_layout.title {
                return title.text == "EigenLayer: Deposit Into Strategy";
            }
        }
        false
    });

    assert!(eigenlayer_field.is_some(), "Should have EigenLayer deposit field");
}

#[test]
fn test_eigenlayer_print_tx() {
    use alloy_consensus::SignableTransaction;
    
    let input_data = hex::decode("e7a050aa00000000000000000000000093c4b944d05dfe6df7645a86cd2206016c51564d000000000000000000000000ae7ab96520de3a18e5e111b5eaab095312d7fe8400000000000000000000000000000000000000000000000000028b30699cdc00").unwrap();

    let tx = TypedTransaction::Legacy(TxLegacy {
        chain_id: Some(ChainId::from(1u64)),
        nonce: 12,
        gas_price: 101521211u128,
        gas_limit: 960000,
        to: alloy_primitives::TxKind::Call("0x858646372CC42E1A627fcE94aa7A7033e7CF075A".parse().unwrap()),
        value: U256::ZERO,
        input: Bytes::from(input_data),
    });

    let mut encoded = Vec::new();
    tx.encode_for_signing(&mut encoded);
    let tx_hex = format!("0x{}", hex::encode(&encoded));
    println!("\n\nEigenLayer depositIntoStrategy transaction:");
    println!("{}\n\n", tx_hex);
}
