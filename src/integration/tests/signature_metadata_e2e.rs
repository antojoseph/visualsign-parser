/// End-to-end test for SignatureMetadata with cryptographic signature verification
///
/// This test validates that:
/// 1. A client can create a ParseRequest with ABI/IDL containing cryptographically signed SignatureMetadata
/// 2. The parser receives and processes it correctly
/// 3. The signature can be verified by the parser using the metadata algorithm and public key

use generated::parser::{
    Abi, Chain, EthereumMetadata, IdlType, Idl, Metadata, ParseRequest, SignatureMetadata,
    SolanaMetadata, SolanaIdlType,
};
use sha2::{Sha256, Digest};
use k256::ecdsa::{SigningKey, VerifyingKey, signature::Signer};
use ed25519_dalek::{SigningKey as Ed25519SigningKey, Signer as Ed25519Signer};

/// Hash content using SHA-256
fn hash_content_sha256(content: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// Sign content with secp256k1 (Ethereum-style)
fn sign_with_secp256k1(content: &str) -> (String, String) {
    let signing_key = SigningKey::random(&mut rand::thread_rng());
    let verifying_key = VerifyingKey::from(&signing_key);
    let message_hash = hash_content_sha256(content);

    let signature = signing_key.sign(&message_hash);
    let signature_hex = format!("{}", hex::encode(signature.to_bytes()));
    let public_key_hex = format!("{}", hex::encode(verifying_key.to_encoded_point(false).as_bytes()));

    (signature_hex, public_key_hex)
}

/// Sign content with ed25519 (Solana-style)
fn sign_with_ed25519(content: &str) -> (String, String) {
    let mut csprng = rand::thread_rng();
    let signing_key = Ed25519SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    let message_hash = hash_content_sha256(content);

    let signature = signing_key.sign(&message_hash);
    let signature_hex = format!("{}", hex::encode(signature.to_bytes()));
    let public_key_hex = format!("{}", hex::encode(verifying_key.to_bytes()));

    (signature_hex, public_key_hex)
}

/// Verify secp256k1 signature
fn verify_secp256k1(content: &str, signature_hex: &str, public_key_hex: &str) -> Result<(), String> {
    use k256::ecdsa::signature::Verifier;
    use k256::EncodedPoint;

    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| format!("Failed to decode signature: {}", e))?;
    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|e| format!("Failed to decode public key: {}", e))?;

    let encoded_point = EncodedPoint::from_bytes(&public_key_bytes)
        .map_err(|e| format!("Failed to parse public key: {}", e))?;
    let verifying_key = VerifyingKey::from_encoded_point(&encoded_point)
        .map_err(|e| format!("Failed to create verifying key: {}", e))?;

    let signature = k256::ecdsa::Signature::from_bytes(signature_bytes.as_slice().into())
        .map_err(|e| format!("Failed to parse signature: {}", e))?;

    let message_hash = hash_content_sha256(content);
    verifying_key.verify(&message_hash, &signature)
        .map_err(|e| format!("Signature verification failed: {}", e))?;

    Ok(())
}

/// Verify ed25519 signature
fn verify_ed25519(content: &str, signature_hex: &str, public_key_hex: &str) -> Result<(), String> {
    use ed25519_dalek::Verifier;

    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| format!("Failed to decode signature: {}", e))?;
    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|e| format!("Failed to decode public key: {}", e))?;

    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
        &public_key_bytes.try_into().map_err(|_| "Invalid public key length")?
    ).map_err(|e| format!("Failed to create verifying key: {}", e))?;

    let signature = ed25519_dalek::Signature::from_bytes(
        &signature_bytes.try_into().map_err(|_| "Invalid signature length")?
    );

    let message_hash = hash_content_sha256(content);
    verifying_key.verify(&message_hash, &signature)
        .map_err(|e| format!("Signature verification failed: {}", e))?;

    Ok(())
}

fn verify_signature_metadata(content: &str, sig_metadata: &SignatureMetadata, public_key: &str) -> Result<(), String> {
    // Extract algorithm from metadata
    let algorithm = sig_metadata
        .metadata
        .iter()
        .find(|m| m.key == "algorithm")
        .map(|m| m.value.clone())
        .ok_or("Missing algorithm in signature metadata")?;

    match algorithm.as_str() {
        "secp256k1" => verify_secp256k1(content, &sig_metadata.value, public_key),
        "ed25519" => verify_ed25519(content, &sig_metadata.value, public_key),
        _ => Err(format!("Unsupported algorithm: {}", algorithm)),
    }
}

#[test]
fn test_ethereum_abi_with_secp256k1_signature() {
    // Simulate a client creating a signed ABI using secp256k1 (Ethereum-style)
    let abi_json = r#"[{"type":"function","name":"transfer","inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}],"outputs":[{"name":"","type":"bool"}]}]"#;

    // Client signs the ABI
    let (signature_hex, public_key_hex) = sign_with_secp256k1(abi_json);

    // Client creates SignatureMetadata with algorithm and public key
    let signature_metadata = SignatureMetadata {
        value: signature_hex.clone(),
        metadata: vec![
            Metadata {
                key: "algorithm".to_string(),
                value: "secp256k1".to_string(),
            },
            Metadata {
                key: "public_key".to_string(),
                value: public_key_hex.clone(),
            },
            Metadata {
                key: "issuer".to_string(),
                value: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            },
            Metadata {
                key: "timestamp".to_string(),
                value: "1699779600".to_string(),
            },
        ],
    };

    // Create Abi with signature
    let abi = Abi {
        value: abi_json.to_string(),
        signature: Some(signature_metadata.clone()),
    };

    // Create ParseRequest with EthereumMetadata containing signed ABI
    let ethereum_metadata = EthereumMetadata { abi: Some(abi) };
    let parse_request = ParseRequest {
        unsigned_payload: "0x".to_string(),
        chain: Chain::Ethereum as i32,
        chain_metadata: Some(generated::parser::parse_request::ChainMetadata::Ethereum(
            ethereum_metadata,
        )),
    };

    // Verify the request was created correctly
    assert_eq!(
        parse_request.chain,
        Chain::Ethereum as i32,
        "Chain should be Ethereum"
    );

    // Simulate parser receiving and verifying the cryptographic signature
    if let Some(generated::parser::parse_request::ChainMetadata::Ethereum(eth_meta)) =
        parse_request.chain_metadata
    {
        if let Some(abi_data) = eth_meta.abi {
            if let Some(sig_meta) = abi_data.signature {
                let verification_result = verify_signature_metadata(&abi_data.value, &sig_meta, &public_key_hex);
                assert!(
                    verification_result.is_ok(),
                    "Signature verification failed: {:?}",
                    verification_result.err()
                );
                println!(
                    "✓ Ethereum ABI signature verified with secp256k1"
                );
            }
        }
    }
}

#[test]
fn test_solana_idl_with_ed25519_signature() {
    // Simulate a client creating a signed IDL using ed25519 (Solana-style)
    let idl_json = r#"{"version":"0.1.0","name":"example_program","instructions":[{"name":"initialize","accounts":[],"args":[]}]}"#;

    // Client signs the IDL
    let (signature_hex, public_key_hex) = sign_with_ed25519(idl_json);

    // Client creates SignatureMetadata with algorithm and public key
    let signature_metadata = SignatureMetadata {
        value: signature_hex.clone(),
        metadata: vec![
            Metadata {
                key: "algorithm".to_string(),
                value: "ed25519".to_string(),
            },
            Metadata {
                key: "public_key".to_string(),
                value: public_key_hex.clone(),
            },
            Metadata {
                key: "issuer".to_string(),
                value: "ExampleProgramAuthority111111111111111111111".to_string(),
            },
            Metadata {
                key: "timestamp".to_string(),
                value: "1699779600".to_string(),
            },
        ],
    };

    // Create Idl with signature
    let idl = Idl {
        value: idl_json.to_string(),
        idl_type: SolanaIdlType::Anchor as i32,
        idl_version: Some("0.30.0".to_string()),
        signature: Some(signature_metadata.clone()),
    };

    // Create ParseRequest with SolanaMetadata containing signed IDL
    let solana_metadata = SolanaMetadata { idl: Some(idl) };
    let parse_request = ParseRequest {
        unsigned_payload: "0x".to_string(),
        chain: Chain::Solana as i32,
        chain_metadata: Some(generated::parser::parse_request::ChainMetadata::Solana(
            solana_metadata,
        )),
    };

    // Verify the request was created correctly
    assert_eq!(
        parse_request.chain,
        Chain::Solana as i32,
        "Chain should be Solana"
    );

    // Simulate parser receiving and verifying the cryptographic signature
    if let Some(generated::parser::parse_request::ChainMetadata::Solana(solana_meta)) =
        parse_request.chain_metadata
    {
        if let Some(idl_data) = solana_meta.idl {
            assert_eq!(
                idl_data.idl_type,
                SolanaIdlType::Anchor as i32,
                "IDL type should be Anchor"
            );
            assert_eq!(
                idl_data.idl_version,
                Some("0.30.0".to_string()),
                "IDL version should be 0.30.0"
            );

            if let Some(sig_meta) = idl_data.signature {
                let verification_result = verify_signature_metadata(&idl_data.value, &sig_meta, &public_key_hex);
                assert!(
                    verification_result.is_ok(),
                    "Signature verification failed: {:?}",
                    verification_result.err()
                );
                println!(
                    "✓ Solana IDL signature verified with ed25519"
                );
            }
        }
    }
}

#[test]
fn test_signature_tampering_detection() {
    // Create signed ABI with secp256k1
    let original_abi = r#"[{"type":"function","name":"transfer"}]"#;
    let (signature_hex, public_key_hex) = sign_with_secp256k1(original_abi);

    let signature_metadata = SignatureMetadata {
        value: signature_hex,
        metadata: vec![
            Metadata {
                key: "algorithm".to_string(),
                value: "secp256k1".to_string(),
            },
            Metadata {
                key: "public_key".to_string(),
                value: public_key_hex.clone(),
            },
        ],
    };

    // Create request with original ABI
    let abi = Abi {
        value: original_abi.to_string(),
        signature: Some(signature_metadata.clone()),
    };

    let ethereum_metadata = EthereumMetadata { abi: Some(abi) };
    let parse_request = ParseRequest {
        unsigned_payload: "0x".to_string(),
        chain: Chain::Ethereum as i32,
        chain_metadata: Some(generated::parser::parse_request::ChainMetadata::Ethereum(
            ethereum_metadata,
        )),
    };

    // Now verify with tampered ABI
    let tampered_abi = r#"[{"type":"function","name":"approve"}]"#;

    if let Some(generated::parser::parse_request::ChainMetadata::Ethereum(eth_meta)) =
        parse_request.chain_metadata
    {
        if let Some(abi_data) = eth_meta.abi {
            if let Some(sig_meta) = abi_data.signature {
                // This should fail because we're verifying tampered content
                let verification_result = verify_signature_metadata(tampered_abi, &sig_meta, &public_key_hex);
                assert!(
                    verification_result.is_err(),
                    "Tampering should be detected!"
                );
                println!(
                    "✓ Tampering detected: {:?}",
                    verification_result.err()
                );
            }
        }
    }
}

#[test]
fn test_metadata_extensibility() {
    // Create signed IDL with ed25519
    let idl_json = r#"{"version":"0.1.0"}"#;
    let (signature_hex, public_key_hex) = sign_with_ed25519(idl_json);

    let mut signature_metadata = SignatureMetadata {
        value: signature_hex,
        metadata: vec![
            Metadata {
                key: "algorithm".to_string(),
                value: "ed25519".to_string(),
            },
            Metadata {
                key: "public_key".to_string(),
                value: public_key_hex.clone(),
            },
        ],
    };

    // Create IDL with minimal metadata
    let mut idl = Idl {
        value: idl_json.to_string(),
        idl_type: SolanaIdlType::Anchor as i32,
        idl_version: Some("0.1.0".to_string()),
        signature: Some(signature_metadata.clone()),
    };

    // Verify original signature works
    if let Some(sig_meta) = &idl.signature {
        let verification = verify_signature_metadata(&idl.value, sig_meta, &public_key_hex);
        assert!(verification.is_ok(), "Original signature should verify");
    }

    // Now add additional metadata without re-signing
    if let Some(sig_meta) = &mut idl.signature {
        sig_meta.metadata.push(Metadata {
            key: "certification_url".to_string(),
            value: "https://example.com/cert".to_string(),
        });
        sig_meta.metadata.push(Metadata {
            key: "version_source".to_string(),
            value: "anchor-0.30.0".to_string(),
        });
    }

    // Verify signature still works after adding metadata
    if let Some(sig_meta) = &idl.signature {
        let verification = verify_signature_metadata(&idl.value, sig_meta, &public_key_hex);
        assert!(
            verification.is_ok(),
            "Signature should remain valid after adding metadata"
        );
        println!(
            "✓ Metadata extended from {} to {} fields without re-signing",
            2,
            sig_meta.metadata.len()
        );
    }
}
