use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;

/// Fetches Jupiter swap transactions from Helius API
pub struct TransactionFetcher {
    helius_api_key: String,
    jupiter_program_id: Pubkey,
    client: reqwest::Client,
}

/// Transaction data fetched from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction: EncodedTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedTransaction {
    pub message: serde_json::Value,
    pub signatures: Vec<String>,
}

/// Helius API response for transaction search
#[derive(Debug, Deserialize)]
struct HeliusResponse {
    result: Vec<HeliusTransaction>,
}

#[derive(Debug, Deserialize)]
struct HeliusTransaction {
    signature: String,
    slot: u64,
    #[serde(rename = "blockTime")]
    block_time: Option<i64>,
    transaction: serde_json::Value,
}

impl TransactionFetcher {
    /// Create a new transaction fetcher
    pub fn new(helius_api_key: impl Into<String>) -> Self {
        let jupiter_program_id =
            Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")
                .expect("Invalid Jupiter program ID");

        Self {
            helius_api_key: helius_api_key.into(),
            jupiter_program_id,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch recent Jupiter swap transactions
    pub async fn fetch_recent_swaps(
        &self,
        input_mint: &str,
        output_mint: &str,
        limit: usize,
    ) -> Result<Vec<TransactionData>> {
        tracing::info!(
            "Fetching up to {} transactions for pair {}-{}",
            limit,
            input_mint,
            output_mint
        );

        // Use Helius enhanced transactions API
        let url = format!(
            "https://api.helius.xyz/v0/transactions?api-key={}",
            self.helius_api_key
        );

        let request_body = serde_json::json!({
            "query": {
                "accounts": [self.jupiter_program_id.to_string()],
                "limit": limit,
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Helius API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Helius API returned error {}: {}", status, error_text);
        }

        let helius_response: HeliusResponse = response
            .json()
            .await
            .context("Failed to parse Helius API response")?;

        // Convert to our format and filter by token pair
        let transactions: Vec<TransactionData> = helius_response
            .result
            .into_iter()
            .filter_map(|tx| {
                // Basic filter - in practice would check token transfers
                Some(TransactionData {
                    signature: tx.signature,
                    slot: tx.slot,
                    block_time: tx.block_time,
                    transaction: EncodedTransaction {
                        message: tx.transaction,
                        signatures: vec![],
                    },
                })
            })
            .take(limit)
            .collect();

        tracing::info!("Fetched {} transactions", transactions.len());

        Ok(transactions)
    }

    /// Fetch a specific transaction by signature
    pub async fn fetch_transaction(
        &self,
        signature: &str,
    ) -> Result<TransactionData> {
        tracing::info!("Fetching transaction {}", signature);

        let sig = Signature::from_str(signature)
            .context("Invalid transaction signature")?;

        let url = format!(
            "https://api.helius.xyz/v0/transactions/{}?api-key={}",
            sig, self.helius_api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to Helius API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Helius API returned error {}: {}", status, error_text);
        }

        let tx: HeliusTransaction = response
            .json()
            .await
            .context("Failed to parse Helius API response")?;

        Ok(TransactionData {
            signature: tx.signature,
            slot: tx.slot,
            block_time: tx.block_time,
            transaction: EncodedTransaction {
                message: tx.transaction,
                signatures: vec![],
            },
        })
    }

    /// Check if Helius API key is available
    pub fn is_available() -> bool {
        std::env::var("HELIUS_API_KEY").is_ok()
    }

    /// Create fetcher from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("HELIUS_API_KEY")
            .context("HELIUS_API_KEY environment variable not set")?;

        Ok(Self::new(api_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetcher_creation() {
        let fetcher = TransactionFetcher::new("test_key");
        assert_eq!(fetcher.helius_api_key, "test_key");
        assert_eq!(
            fetcher.jupiter_program_id.to_string(),
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"
        );
    }

    #[tokio::test]
    #[ignore] // Requires HELIUS_API_KEY
    async fn test_fetch_from_env() {
        if !TransactionFetcher::is_available() {
            println!("Skipping: HELIUS_API_KEY not set");
            return;
        }

        let fetcher = TransactionFetcher::from_env().unwrap();
        let transactions = fetcher
            .fetch_recent_swaps(
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                5,
            )
            .await
            .unwrap();

        assert!(!transactions.is_empty());
        println!("Fetched {} transactions", transactions.len());
    }
}
