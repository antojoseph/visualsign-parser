//! System program preset for Solana

use crate::core::{InstructionVisualizer, SolanaIntegrationConfig, SolanaIntegrationConfigData, VisualizerContext, VisualizerKind};
use std::collections::HashMap;
use solana_program::system_instruction::SystemInstruction;
use visualsign::{AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon};
use visualsign::errors::VisualSignError;

// Create a static instance that we can reference
static SYSTEM_CONFIG: SystemConfig = SystemConfig;

pub struct SystemConfig;

impl SolanaIntegrationConfig for SystemConfig {
    fn new() -> Self {
        Self
    }

    fn data(&self) -> &SolanaIntegrationConfigData {
        static DATA: std::sync::OnceLock<SolanaIntegrationConfigData> = std::sync::OnceLock::new();
        DATA.get_or_init(|| {
            let mut programs = HashMap::new();
            let mut system_instructions = HashMap::new();
            system_instructions.insert("*", vec!["*"]);
            programs.insert("11111111111111111111111111111111", system_instructions);
            SolanaIntegrationConfigData { programs }
        })
    }
}

pub struct SystemVisualizer;

impl InstructionVisualizer for SystemVisualizer {
    fn visualize_tx_commands(
        &self,
        context: &VisualizerContext,
    ) -> Result<AnnotatedPayloadField, VisualSignError> {
        let instruction = context.current_instruction()
            .ok_or_else(|| VisualSignError::MissingData("No instruction found".into()))?;

        // Try to parse as system instruction
        let system_instruction = bincode::deserialize::<SystemInstruction>(&instruction.data)
            .map_err(|e| VisualSignError::DecodeError(format!("Failed to parse system instruction: {}", e)))?;

        let instruction_text = format_system_instruction(&system_instruction);

        Ok(AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    label: "System Instruction".to_string(),
                    fallback_text: instruction_text.clone(),
                },
                text_v2: visualsign::SignablePayloadFieldTextV2 {
                    text: instruction_text,
                },
            },
        })
    }

    fn get_config(&self) -> Option<&dyn SolanaIntegrationConfig> {
        Some(&SYSTEM_CONFIG)
    }

    fn kind(&self) -> VisualizerKind {
        VisualizerKind::Payments("System")
    }
}

fn format_system_instruction(instruction: &SystemInstruction) -> String {
    match instruction {
        SystemInstruction::CreateAccount { owner, .. } => format!("Create Account (owner: {})", owner),
        SystemInstruction::Assign { owner } => format!("Assign (owner: {})", owner),
        SystemInstruction::Transfer { lamports } => format!("Transfer {} lamports", lamports),
        SystemInstruction::CreateAccountWithSeed { owner, .. } => format!("Create Account With Seed (owner: {})", owner),
        SystemInstruction::AdvanceNonceAccount => "Advance Nonce Account".to_string(),
        SystemInstruction::WithdrawNonceAccount(lamports) => format!("Withdraw Nonce Account ({} lamports)", lamports),
        SystemInstruction::InitializeNonceAccount(_) => "Initialize Nonce Account".to_string(),
        SystemInstruction::AuthorizeNonceAccount(_) => "Authorize Nonce Account".to_string(),
        SystemInstruction::Allocate { space } => format!("Allocate (space: {})", space),
        SystemInstruction::AllocateWithSeed { owner, .. } => format!("Allocate With Seed (owner: {})", owner),
        SystemInstruction::AssignWithSeed { base, seed, owner } => format!("Assign With Seed (base: {}, seed: {}, owner: {})", base, seed, owner),
        SystemInstruction::TransferWithSeed { from_owner, .. } => format!("Transfer With Seed (from_owner: {})", from_owner),
        SystemInstruction::UpgradeNonceAccount => "Upgrade Nonce Account".to_string(),
    }
}
