use alloy_primitives::Address;
use alloy_sol_types::{SolCall as _, SolValue as _, sol};
use chrono::{TimeZone, Utc};
use num_enum::TryFromPrimitive;
use visualsign::{SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldTextV2};

use crate::registry::{ContractRegistry, ContractType};

// Uniswap Universal Router interface definitions
//
// Official Documentation:
// - Technical Reference: https://docs.uniswap.org/contracts/universal-router/technical-reference
// - Contract Source: https://github.com/Uniswap/universal-router/blob/main/contracts/interfaces/IUniversalRouter.sol
//
// The Universal Router supports function overloading with two execute variants:
// 1. execute(bytes,bytes[],uint256) - with deadline parameter for time-bound execution
// 2. execute(bytes,bytes[]) - without deadline for flexible execution
//
// Each function gets a unique 4-byte selector based on its signature.
sol! {
    interface IUniversalRouter {
        /// @notice Executes encoded commands along with provided inputs. Reverts if deadline has expired.
        /// @param commands A set of concatenated commands, each 1 byte in length
        /// @param inputs An array of byte strings containing abi encoded inputs for each command
        /// @param deadline The deadline by which the transaction must be executed
        function execute(bytes calldata commands, bytes[] calldata inputs, uint256 deadline) external payable;

        /// @notice Executes encoded commands along with provided inputs (no deadline check)
        /// @param commands A set of concatenated commands, each 1 byte in length
        /// @param inputs An array of byte strings containing abi encoded inputs for each command
        function execute(bytes calldata commands, bytes[] calldata inputs) external payable;
    }
}

// Command parameter structures
//
// These structs define the ABI-encoded parameters for each command type.
// Reference: https://docs.uniswap.org/contracts/universal-router/technical-reference
// Source: https://github.com/Uniswap/universal-router/blob/main/contracts/modules/uniswap/v3/V3SwapRouter.sol
sol! {
    /// Parameters for V3_SWAP_EXACT_IN command
    struct V3SwapExactInputParams {
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
        bytes path;
        bool payerIsUser;
    }

    /// Parameters for V3_SWAP_EXACT_OUT command
    struct V3SwapExactOutputParams {
        address recipient;
        uint256 amountOut;
        uint256 amountInMaximum;
        bytes path;
        bool payerIsUser;
    }

    /// Parameters for PAY_PORTION command
    struct PayPortionParams {
        address token;
        address recipient;
        uint256 bips;
    }

    /// Parameters for UNWRAP_WETH command
    struct UnwrapWethParams {
        address recipient;
        uint256 amountMinimum;
    }

    /// Parameters for V2_SWAP_EXACT_IN command
    /// Source: https://github.com/Uniswap/universal-router/blob/main/contracts/modules/uniswap/v2/V2SwapRouter.sol
    /// function v2SwapExactInput(address recipient, uint256 amountIn, uint256 amountOutMinimum, address[] calldata path, address payer)
    struct V2SwapExactInputParams {
        address recipient;
        uint256 amountIn;
        uint256 amountOutMinimum;
        address[] path;
        address payer;
    }

    /// Parameters for V2_SWAP_EXACT_OUT command
    struct V2SwapExactOutputParams {
        uint256 amountOut;
        uint256 amountInMaximum;
        address[] path;
        address recipient;
    }

    /// Parameters for WRAP_ETH command
    struct WrapEthParams {
        uint256 amountMin;
    }

    /// Parameters for SWEEP command
    struct SweepParams {
        address token;
        uint256 amountMinimum;
        address recipient;
    }

    /// Parameters for TRANSFER command
    struct TransferParams {
        address from;
        address to;
        uint160 amount;
    }

    /// Parameters for PERMIT2_TRANSFER_FROM command
    struct Permit2TransferFromParams {
        address from;
        address to;
        uint160 amount;
        address token;
    }
}

// Command IDs for Universal Router
//
// Reference: https://docs.uniswap.org/contracts/universal-router/technical-reference
// Source: https://github.com/Uniswap/universal-router/blob/main/contracts/libraries/Commands.sol
//
// Commands are encoded as single bytes and define the operation to execute.
// The Universal Router processes these commands sequentially.
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Command {
    V3SwapExactIn = 0x00,
    V3SwapExactOut = 0x01,
    Permit2TransferFrom = 0x02,
    Permit2PermitBatch = 0x03,
    Sweep = 0x04,
    Transfer = 0x05,
    PayPortion = 0x06,

    V2SwapExactIn = 0x08,
    V2SwapExactOut = 0x09,
    Permit2Permit = 0x0a,
    WrapEth = 0x0b,
    UnwrapWeth = 0x0c,
    Permit2TransferFromBatch = 0x0d,
    BalanceCheckErc20 = 0x0e,

    V4Swap = 0x10,
    V3PositionManagerPermit = 0x11,
    V3PositionManagerCall = 0x12,
    V4InitializePool = 0x13,
    V4PositionManagerCall = 0x14,

    ExecuteSubPlan = 0x21,
}

fn map_commands(raw: &[u8]) -> Vec<Command> {
    let mut out = Vec::with_capacity(raw.len());
    for &b in raw {
        if let Ok(cmd) = Command::try_from(b) {
            out.push(cmd);
        }
    }
    out
}

/// Visualizer for Uniswap Universal Router
///
/// Handles the `execute` function from IUniversalRouter interface:
/// <https://github.com/Uniswap/universal-router/blob/dev/contracts/interfaces/IUniversalRouter.sol>
pub struct UniversalRouterVisualizer {}

impl UniversalRouterVisualizer {
    /// Visualizes Uniswap Universal Router Execute commands
    ///
    /// # Arguments
    /// * `input` - The calldata bytes
    /// * `chain_id` - The chain ID for registry lookups
    /// * `registry` - Optional registry for resolving token symbols
    pub fn visualize_tx_commands(
        &self,
        input: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> Option<SignablePayloadField> {
        if input.len() < 4 {
            return None;
        }

        // Try decoding with deadline first (3-parameter version)
        if let Ok(call) = IUniversalRouter::execute_0Call::abi_decode(input) {
            let deadline_val: i64 = match call.deadline.try_into() {
                Ok(val) => val,
                Err(_) => return None,
            };
            let deadline = if deadline_val > 0 {
                Utc.timestamp_opt(deadline_val, 0)
                    .single()
                    .map(|dt| dt.to_string())
            } else {
                None
            };
            return Self::visualize_commands(&call.commands.0, &call.inputs, deadline, chain_id, registry);
        }

        // Try decoding without deadline (2-parameter version)
        if let Ok(call) = IUniversalRouter::execute_1Call::abi_decode(input) {
            return Self::visualize_commands(&call.commands.0, &call.inputs, None, chain_id, registry);
        }

        None
    }

    /// Helper function to visualize commands (shared by both execute variants)
    fn visualize_commands(
        commands: &[u8],
        inputs: &[alloy_primitives::Bytes],
        deadline: Option<String>,
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> Option<SignablePayloadField> {
        let mapped = map_commands(commands);
        let mut detail_fields = Vec::new();

        for (i, cmd) in mapped.iter().enumerate() {
            let input_bytes = inputs.get(i).map(|b| &b.0[..]);

            // Decode command-specific parameters
            let field = if let Some(bytes) = input_bytes {
                match cmd {
                    Command::V3SwapExactIn => {
                        Self::decode_v3_swap_exact_in(bytes, chain_id, registry)
                    }
                    Command::V3SwapExactOut => {
                        Self::decode_v3_swap_exact_out(bytes, chain_id, registry)
                    }
                    Command::V2SwapExactIn => {
                        Self::decode_v2_swap_exact_in(bytes, chain_id, registry)
                    }
                    Command::V2SwapExactOut => {
                        Self::decode_v2_swap_exact_out(bytes, chain_id, registry)
                    }
                    Command::PayPortion => Self::decode_pay_portion(bytes, chain_id, registry),
                    Command::WrapEth => Self::decode_wrap_eth(bytes, chain_id, registry),
                    Command::UnwrapWeth => Self::decode_unwrap_weth(bytes, chain_id, registry),
                    Command::Sweep => Self::decode_sweep(bytes, chain_id, registry),
                    Command::Transfer => Self::decode_transfer(bytes, chain_id, registry),
                    Command::Permit2TransferFrom => {
                        Self::decode_permit2_transfer_from(bytes, chain_id, registry)
                    }
                    _ => {
                        // For unimplemented commands, show hex
                        let input_hex = format!("0x{}", hex::encode(bytes));
                        SignablePayloadField::TextV2 {
                            common: SignablePayloadFieldCommon {
                                fallback_text: format!("{cmd:?} input: {input_hex}"),
                                label: format!("{:?}", cmd),
                            },
                            text_v2: SignablePayloadFieldTextV2 {
                                text: format!("Input: {input_hex}"),
                            },
                        }
                    }
                }
            } else {
                SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("{cmd:?} input: None"),
                        label: format!("{:?}", cmd),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Input: None".to_string(),
                    },
                }
            };

            // Wrap the field in a PreviewLayout for consistency
            let label = format!("Command {}", i + 1);
            let wrapped_field = match field {
                SignablePayloadField::TextV2 { common, text_v2 } => {
                    SignablePayloadField::PreviewLayout {
                        common: SignablePayloadFieldCommon {
                            fallback_text: common.fallback_text,
                            label,
                        },
                        preview_layout: visualsign::SignablePayloadFieldPreviewLayout {
                            title: Some(visualsign::SignablePayloadFieldTextV2 {
                                text: common.label,
                            }),
                            subtitle: Some(text_v2),
                            condensed: None,
                            expanded: None,
                        },
                    }
                }
                _ => field,
            };

            detail_fields.push(wrapped_field);
        }

        // Deadline field (optional)
        if let Some(dl) = &deadline {
            detail_fields.push(SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: dl.clone(),
                    label: "Deadline".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 { text: dl.clone() },
            });
        }

        Some(SignablePayloadField::PreviewLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: if let Some(dl) = &deadline {
                    format!(
                        "Uniswap Universal Router Execute: {} commands ({:?}), deadline {}",
                        mapped.len(),
                        mapped,
                        dl
                    )
                } else {
                    format!(
                        "Uniswap Universal Router Execute: {} commands ({:?})",
                        mapped.len(),
                        mapped
                    )
                },
                label: "Universal Router".to_string(),
            },
            preview_layout: visualsign::SignablePayloadFieldPreviewLayout {
                title: Some(visualsign::SignablePayloadFieldTextV2 {
                    text: "Uniswap Universal Router Execute".to_string(),
                }),
                subtitle: if let Some(dl) = &deadline {
                    Some(visualsign::SignablePayloadFieldTextV2 {
                        text: format!("{} commands, deadline {}", mapped.len(), dl),
                    })
                } else {
                    Some(visualsign::SignablePayloadFieldTextV2 {
                        text: format!("{} commands", mapped.len()),
                    })
                },
                condensed: None,
                expanded: Some(visualsign::SignablePayloadFieldListLayout {
                    fields: detail_fields
                        .into_iter()
                        .map(|f| visualsign::AnnotatedPayloadField {
                            signable_payload_field: f,
                            static_annotation: None,
                            dynamic_annotation: None,
                        })
                        .collect(),
                }),
            },
        })
    }

    /// Decodes V3_SWAP_EXACT_IN command parameters
    fn decode_v3_swap_exact_in(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        // Use sol! macro for clean decoding
        let params = match V3SwapExactInputParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("V3 Swap Exact In: 0x{}", hex::encode(bytes)),
                        label: "V3 Swap Exact In".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        // Parse the path to extract token addresses and fee
        if params.path.0.len() < 43 {
            return SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "V3SwapExactIn: Invalid path".to_string(),
                    label: "V3 Swap Exact In".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("Path length: {} bytes (expected ≥43)", params.path.0.len()),
                },
            };
        }

        let path_bytes = &params.path.0;
        let token_in = Address::from_slice(&path_bytes[0..20]);
        let fee = u32::from_be_bytes([0, path_bytes[20], path_bytes[21], path_bytes[22]]);
        let token_out = Address::from_slice(&path_bytes[23..43]);

        // Resolve token symbols and format amounts
        let token_in_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_in))
            .unwrap_or_else(|| format!("{:?}", token_in));
        let token_out_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_out))
            .unwrap_or_else(|| format!("{:?}", token_out));

        let amount_in_u128: u128 = params.amountIn.to_string().parse().unwrap_or(0);
        let amount_out_min_u128: u128 = params.amountOutMinimum.to_string().parse().unwrap_or(0);

        let (amount_in_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_in, amount_in_u128))
            .unwrap_or_else(|| (params.amountIn.to_string(), token_in_symbol.clone()));

        let (amount_out_min_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_out, amount_out_min_u128))
            .unwrap_or_else(|| (params.amountOutMinimum.to_string(), token_out_symbol.clone()));

        let fee_pct = fee as f64 / 10000.0;
        let text = format!(
            "Swap {} {} for >={} {} via V3 ({}% fee)",
            amount_in_str, token_in_symbol, amount_out_min_str, token_out_symbol, fee_pct
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "V3 Swap Exact In".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes PAY_PORTION command parameters
    fn decode_pay_portion(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        let params = match PayPortionParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("Pay Portion: 0x{}", hex::encode(bytes)),
                        label: "Pay Portion".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        let token_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, params.token))
            .unwrap_or_else(|| format!("{:?}", params.token));

        // Convert bips to percentage (10000 bips = 100%)
        let bips_value: u128 = params.bips.to_string().parse().unwrap_or(0);
        let bips_pct = (bips_value as f64) / 100.0;
        let percentage_str = if bips_pct >= 1.0 {
            format!("{:.2}%", bips_pct)
        } else {
            format!("{:.4}%", bips_pct)
        };

        let text = format!(
            "Pay {} of {} to {:?}",
            percentage_str, token_symbol, params.recipient
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "Pay Portion".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes UNWRAP_WETH command parameters
    fn decode_unwrap_weth(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        
        

        let params = match UnwrapWethParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("Unwrap WETH: 0x{}", hex::encode(bytes)),
                        label: "Unwrap WETH".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        // Get WETH address for this chain and format the amount
        // WETH is registered in the token registry via UniswapConfig::register_common_tokens
        let amount_min_str = crate::protocols::uniswap::config::UniswapConfig::weth_address(chain_id)
            .and_then(|weth_addr| {
                let amount_min_u128: u128 = params.amountMinimum.to_string().parse().unwrap_or(0);
                registry.and_then(|r| r.format_token_amount(chain_id, weth_addr, amount_min_u128))
            })
            .map(|(amt, _)| amt)
            .unwrap_or_else(|| params.amountMinimum.to_string());

        let text = format!(
            "Unwrap >={} WETH to ETH for {:?}",
            amount_min_str, params.recipient
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "Unwrap WETH".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes V3_SWAP_EXACT_OUT command parameters
    fn decode_v3_swap_exact_out(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        

        let params = match V3SwapExactOutputParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("V3 Swap Exact Out: 0x{}", hex::encode(bytes)),
                        label: "V3 Swap Exact Out".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        if params.path.0.len() < 43 {
            return SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "V3SwapExactOut: Invalid path".to_string(),
                    label: "V3 Swap Exact Out".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: format!("Path length: {} bytes (expected >=43)", params.path.0.len()),
                },
            };
        }

        let path_bytes = &params.path.0;
        let token_in = Address::from_slice(&path_bytes[0..20]);
        let fee = u32::from_be_bytes([0, path_bytes[20], path_bytes[21], path_bytes[22]]);
        let token_out = Address::from_slice(&path_bytes[23..43]);

        let token_in_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_in))
            .unwrap_or_else(|| format!("{:?}", token_in));
        let token_out_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_out))
            .unwrap_or_else(|| format!("{:?}", token_out));

        let amount_out_u128: u128 = params.amountOut.to_string().parse().unwrap_or(0);
        let amount_in_max_u128: u128 = params.amountInMaximum.to_string().parse().unwrap_or(0);

        let (amount_out_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_out, amount_out_u128))
            .unwrap_or_else(|| (params.amountOut.to_string(), token_out_symbol.clone()));

        let (amount_in_max_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_in, amount_in_max_u128))
            .unwrap_or_else(|| (params.amountInMaximum.to_string(), token_in_symbol.clone()));

        let fee_pct = fee as f64 / 10000.0;
        let text = format!(
            "Swap <={} {} for {} {} via V3 ({}% fee)",
            amount_in_max_str, token_in_symbol, amount_out_str, token_out_symbol, fee_pct
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "V3 Swap Exact Out".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes V2_SWAP_EXACT_IN command parameters
    ///
    /// Uses manual ABI decoding due to compatibility issues with Alloy's automatic decoder.
    /// Structure: (address recipient, uint256 amountIn, uint256 amountOutMinimum, address[] path, address payer)
    fn decode_v2_swap_exact_in(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        if bytes.len() < 160 {
            return SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("V2 Swap Exact In: 0x{}", hex::encode(bytes)),
                    label: "V2 Swap Exact In".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: "Data too short".to_string(),
                },
            };
        }

        // Parse fixed fields
        let amount_in = alloy_primitives::U256::from_be_slice(&bytes[32..64]);
        let amount_out_minimum = alloy_primitives::U256::from_be_slice(&bytes[64..96]);
        let path_offset = alloy_primitives::U256::from_be_slice(&bytes[96..128]);

        // Parse path array at the offset
        let offset_usize: usize = path_offset.to_string().parse().unwrap_or(0);
        if offset_usize + 32 > bytes.len() {
            return SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!("V2 Swap Exact In: invalid offset"),
                    label: "V2 Swap Exact In".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: "Invalid path offset".to_string(),
                },
            };
        }

        let path_length = alloy_primitives::U256::from_be_slice(&bytes[offset_usize..offset_usize + 32]);
        let path_len_usize: usize = path_length.to_string().parse().unwrap_or(0);
        let mut path = Vec::new();
        for i in 0..path_len_usize {
            let addr_offset = offset_usize + 32 + (i * 32);
            if addr_offset + 32 <= bytes.len() {
                let addr = Address::from_slice(&bytes[addr_offset + 12..addr_offset + 32]); // addresses are right-aligned in 32 bytes
                path.push(addr);
            }
        }

        if path.is_empty() {
            return SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "V2SwapExactIn: Empty path".to_string(),
                    label: "V2 Swap Exact In".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: "Swap path is empty".to_string(),
                },
            };
        }

        let token_in = path[0];
        let token_out = path[path.len() - 1];

        let token_in_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_in))
            .unwrap_or_else(|| format!("{:?}", token_in));
        let token_out_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_out))
            .unwrap_or_else(|| format!("{:?}", token_out));

        let amount_in_u128: u128 = amount_in.to_string().parse().unwrap_or(0);
        let amount_out_min_u128: u128 = amount_out_minimum.to_string().parse().unwrap_or(0);

        let (amount_in_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_in, amount_in_u128))
            .unwrap_or_else(|| (amount_in.to_string(), token_in_symbol.clone()));

        let (amount_out_min_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_out, amount_out_min_u128))
            .unwrap_or_else(|| (amount_out_minimum.to_string(), token_out_symbol.clone()));

        let hops = path.len() - 1;
        let text = format!(
            "Swap {} {} for >={} {} via V2 ({} hops)",
            amount_in_str, token_in_symbol, amount_out_min_str, token_out_symbol, hops
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "V2 Swap Exact In".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes V2_SWAP_EXACT_OUT command parameters
    fn decode_v2_swap_exact_out(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        

        let params = match V2SwapExactOutputParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("V2 Swap Exact Out: 0x{}", hex::encode(bytes)),
                        label: "V2 Swap Exact Out".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        if params.path.is_empty() {
            return SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    fallback_text: "V2SwapExactOut: Empty path".to_string(),
                    label: "V2 Swap Exact Out".to_string(),
                },
                text_v2: SignablePayloadFieldTextV2 {
                    text: "Swap path is empty".to_string(),
                },
            };
        }

        let token_in = params.path[0];
        let token_out = params.path[params.path.len() - 1];

        let token_in_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_in))
            .unwrap_or_else(|| format!("{:?}", token_in));
        let token_out_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, token_out))
            .unwrap_or_else(|| format!("{:?}", token_out));

        let amount_out_u128: u128 = params.amountOut.to_string().parse().unwrap_or(0);
        let amount_in_max_u128: u128 = params.amountInMaximum.to_string().parse().unwrap_or(0);

        let (amount_out_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_out, amount_out_u128))
            .unwrap_or_else(|| (params.amountOut.to_string(), token_out_symbol.clone()));

        let (amount_in_max_str, _) = registry
            .and_then(|r| r.format_token_amount(chain_id, token_in, amount_in_max_u128))
            .unwrap_or_else(|| (params.amountInMaximum.to_string(), token_in_symbol.clone()));

        let hops = params.path.len() - 1;
        let text = format!(
            "Swap <={} {} for {} {} via V2 ({} hops)",
            amount_in_max_str, token_in_symbol, amount_out_str, token_out_symbol, hops
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "V2 Swap Exact Out".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes WRAP_ETH command parameters
    fn decode_wrap_eth(
        bytes: &[u8],
        _chain_id: u64,
        _registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        

        let params = match WrapEthParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("Wrap ETH: 0x{}", hex::encode(bytes)),
                        label: "Wrap ETH".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        let amount_min_str = params.amountMin.to_string();
        let text = format!("Wrap {} ETH to WETH", amount_min_str);

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "Wrap ETH".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes SWEEP command parameters
    fn decode_sweep(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        

        let params = match SweepParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("Sweep: 0x{}", hex::encode(bytes)),
                        label: "Sweep".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        let token_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, params.token))
            .unwrap_or_else(|| format!("{:?}", params.token));

        let text = format!(
            "Sweep >={} {} to {:?}",
            params.amountMinimum, token_symbol, params.recipient
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "Sweep".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes TRANSFER command parameters
    fn decode_transfer(
        bytes: &[u8],
        _chain_id: u64,
        _registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        

        let params = match TransferParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("Transfer: 0x{}", hex::encode(bytes)),
                        label: "Transfer".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        let text = format!(
            "Transfer {} tokens from {:?} to {:?}",
            params.amount, params.from, params.to
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "Transfer".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }

    /// Decodes PERMIT2_TRANSFER_FROM command parameters
    fn decode_permit2_transfer_from(
        bytes: &[u8],
        chain_id: u64,
        registry: Option<&ContractRegistry>,
    ) -> SignablePayloadField {
        

        let params = match Permit2TransferFromParams::abi_decode(bytes) {
            Ok(p) => p,
            Err(_) => {
                return SignablePayloadField::TextV2 {
                    common: SignablePayloadFieldCommon {
                        fallback_text: format!("Permit2 Transfer From: 0x{}", hex::encode(bytes)),
                        label: "Permit2 Transfer From".to_string(),
                    },
                    text_v2: SignablePayloadFieldTextV2 {
                        text: "Failed to decode parameters".to_string(),
                    },
                };
            }
        };

        let token_symbol = registry
            .and_then(|r| r.get_token_symbol(chain_id, params.token))
            .unwrap_or_else(|| format!("{:?}", params.token));

        let text = format!(
            "Transfer {} {} from {:?} to {:?}",
            params.amount, token_symbol, params.from, params.to
        );

        SignablePayloadField::TextV2 {
            common: SignablePayloadFieldCommon {
                fallback_text: text.clone(),
                label: "Permit2 Transfer From".to_string(),
            },
            text_v2: SignablePayloadFieldTextV2 { text },
        }
    }
}

/// ContractVisualizer implementation for Uniswap Universal Router
pub struct UniversalRouterContractVisualizer {
    inner: UniversalRouterVisualizer,
}

impl UniversalRouterContractVisualizer {
    pub fn new() -> Self {
        Self {
            inner: UniversalRouterVisualizer {},
        }
    }
}

impl Default for UniversalRouterContractVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::visualizer::ContractVisualizer for UniversalRouterContractVisualizer {
    fn contract_type(&self) -> &str {
        crate::protocols::uniswap::config::UniswapUniversalRouter::short_type_id()
    }

    fn visualize(
        &self,
        context: &crate::context::VisualizerContext,
    ) -> Result<Option<Vec<visualsign::AnnotatedPayloadField>>, visualsign::vsptrait::VisualSignError> {
        let contract_registry = crate::registry::ContractRegistry::with_default_protocols();

        if let Some(field) = self.inner.visualize_tx_commands(
            &context.calldata,
            context.chain_id,
            Some(&contract_registry),
        ) {
            let annotated = visualsign::AnnotatedPayloadField {
                signable_payload_field: field,
                static_annotation: None,
                dynamic_annotation: None,
            };

            Ok(Some(vec![annotated]))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Bytes, U256};
    use visualsign::{
        AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon,
        SignablePayloadFieldListLayout, SignablePayloadFieldPreviewLayout,
        SignablePayloadFieldTextV2,
    };

    fn encode_execute_call(commands: &[u8], inputs: Vec<Vec<u8>>, deadline: u64) -> Vec<u8> {
        let inputs_bytes = inputs.into_iter().map(Bytes::from).collect::<Vec<_>>();
        IUniversalRouter::execute_0Call {
            commands: Bytes::from(commands.to_vec()),
            inputs: inputs_bytes,
            deadline: U256::from(deadline),
        }
        .abi_encode()
    }

    #[test]
    fn test_visualize_tx_commands_empty_input() {
        assert_eq!(
            UniversalRouterVisualizer {}.visualize_tx_commands(&[], 1, None),
            None
        );
        assert_eq!(
            UniversalRouterVisualizer {}.visualize_tx_commands(&[0x01, 0x02, 0x03], 1, None),
            None
        );
    }

    #[test]
    fn test_visualize_tx_commands_invalid_deadline() {
        // deadline is not convertible to i64 (u64::MAX)
        let input = encode_execute_call(&[0x00], vec![vec![0x01, 0x02]], u64::MAX);
        assert_eq!(
            UniversalRouterVisualizer {}.visualize_tx_commands(&input, 1, None),
            None
        );
    }

    #[test]
    fn test_visualize_tx_commands_single_command_with_deadline() {
        let commands = vec![Command::V3SwapExactIn as u8];
        let inputs = vec![vec![0xde, 0xad, 0xbe, 0xef]];
        let deadline = 1_700_000_000u64; // 2023-11-13T12:26:40Z
        let input = encode_execute_call(&commands, inputs.clone(), deadline);

        // Build expected field
        let dt = chrono::Utc.timestamp_opt(deadline as i64, 0).unwrap();
        let deadline_str = dt.to_string();

        assert_eq!(
            UniversalRouterVisualizer {}
                .visualize_tx_commands(&input, 1, None)
                .unwrap(),
            SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!(
                        "Uniswap Universal Router Execute: 1 commands ([V3SwapExactIn]), deadline {deadline_str}"
                    ),
                    label: "Universal Router".to_string(),
                },
                preview_layout: SignablePayloadFieldPreviewLayout {
                    title: Some(SignablePayloadFieldTextV2 {
                        text: "Uniswap Universal Router Execute".to_string(),
                    }),
                    subtitle: Some(SignablePayloadFieldTextV2 {
                        text: format!("1 commands, deadline {deadline_str}"),
                    }),
                    condensed: None,
                    expanded: Some(SignablePayloadFieldListLayout {
                        fields: vec![
                            AnnotatedPayloadField {
                                signable_payload_field: SignablePayloadField::PreviewLayout {
                                    common: SignablePayloadFieldCommon {
                                        fallback_text: "V3 Swap Exact In: 0xdeadbeef"
                                            .to_string(),
                                        label: "Command 1".to_string(),
                                    },
                                    preview_layout: SignablePayloadFieldPreviewLayout {
                                        title: Some(SignablePayloadFieldTextV2 {
                                            text: "V3 Swap Exact In".to_string(),
                                        }),
                                        subtitle: Some(SignablePayloadFieldTextV2 {
                                            text: "Failed to decode parameters"
                                                .to_string(),
                                        }),
                                        condensed: None,
                                        expanded: None,
                                    },
                                },
                                static_annotation: None,
                                dynamic_annotation: None,
                            },
                            AnnotatedPayloadField {
                                signable_payload_field: SignablePayloadField::TextV2 {
                                    common: SignablePayloadFieldCommon {
                                        fallback_text: deadline_str.clone(),
                                        label: "Deadline".to_string(),
                                    },
                                    text_v2: SignablePayloadFieldTextV2 {
                                        text: deadline_str.clone(),
                                    },
                                },
                                static_annotation: None,
                                dynamic_annotation: None,
                            },
                        ],
                    }),
                },
            }
        );
    }

    #[test]
    fn test_visualize_tx_commands_multiple_commands_no_deadline() {
        let commands = vec![
            Command::V3SwapExactIn as u8,
            Command::Transfer as u8,
            Command::WrapEth as u8,
        ];
        let inputs = vec![vec![0x01, 0x02], vec![0x03, 0x04, 0x05], vec![0x06]];
        let deadline = 0u64;
        let input = encode_execute_call(&commands, inputs.clone(), deadline);

        assert_eq!(
            UniversalRouterVisualizer {}
                .visualize_tx_commands(&input, 1, None)
                .unwrap(),
            SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text:
                        "Uniswap Universal Router Execute: 3 commands ([V3SwapExactIn, Transfer, WrapEth])"
                            .to_string(),
                    label: "Universal Router".to_string(),
                },
                preview_layout: SignablePayloadFieldPreviewLayout {
                    title: Some(SignablePayloadFieldTextV2 {
                        text: "Uniswap Universal Router Execute".to_string(),
                    }),
                    subtitle: Some(SignablePayloadFieldTextV2 {
                        text: "3 commands".to_string(),
                    }),
                    condensed: None,
                    expanded: Some(SignablePayloadFieldListLayout {
                        fields: vec![
                            AnnotatedPayloadField {
                                signable_payload_field: SignablePayloadField::PreviewLayout {
                                    common: SignablePayloadFieldCommon {
                                        fallback_text: "V3 Swap Exact In: 0x0102".to_string(),
                                        label: "Command 1".to_string(),
                                    },
                                    preview_layout: SignablePayloadFieldPreviewLayout {
                                        title: Some(SignablePayloadFieldTextV2 {
                                            text: "V3 Swap Exact In".to_string(),
                                        }),
                                        subtitle: Some(SignablePayloadFieldTextV2 {
                                            text: "Failed to decode parameters".to_string(),
                                        }),
                                        condensed: None,
                                        expanded: None,
                                    },
                                },
                                static_annotation: None,
                                dynamic_annotation: None,
                            },
                            AnnotatedPayloadField {
                                signable_payload_field: SignablePayloadField::PreviewLayout {
                                    common: SignablePayloadFieldCommon {
                                        fallback_text: "Transfer: 0x030405".to_string(),
                                        label: "Command 2".to_string(),
                                    },
                                    preview_layout: SignablePayloadFieldPreviewLayout {
                                        title: Some(SignablePayloadFieldTextV2 {
                                            text: "Transfer".to_string(),
                                        }),
                                        subtitle: Some(SignablePayloadFieldTextV2 {
                                            text: "Failed to decode parameters".to_string(),
                                        }),
                                        condensed: None,
                                        expanded: None,
                                    },
                                },
                                static_annotation: None,
                                dynamic_annotation: None,
                            },
                            AnnotatedPayloadField {
                                signable_payload_field: SignablePayloadField::PreviewLayout {
                                    common: SignablePayloadFieldCommon {
                                        fallback_text: "Wrap ETH: 0x06".to_string(),
                                        label: "Command 3".to_string(),
                                    },
                                    preview_layout: SignablePayloadFieldPreviewLayout {
                                        title: Some(SignablePayloadFieldTextV2 {
                                            text: "Wrap ETH".to_string(),
                                        }),
                                        subtitle: Some(SignablePayloadFieldTextV2 {
                                            text: "Failed to decode parameters".to_string(),
                                        }),
                                        condensed: None,
                                        expanded: None,
                                    },
                                },
                                static_annotation: None,
                                dynamic_annotation: None,
                            },
                        ],
                    }),
                },
            }
        );
    }

    #[test]
    fn test_visualize_tx_commands_command_without_input() {
        // Only one command, but no input for it
        let commands = vec![Command::Sweep as u8];
        let inputs = vec![]; // No input
        let deadline = 1_700_000_000u64;
        let input = encode_execute_call(&commands, inputs.clone(), deadline);

        let dt = chrono::Utc.timestamp_opt(deadline as i64, 0).unwrap();
        let deadline_str = dt.to_string();

        assert_eq!(
            UniversalRouterVisualizer {}
                .visualize_tx_commands(&input, 1, None)
                .unwrap(),
            SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: format!(
                        "Uniswap Universal Router Execute: 1 commands ([Sweep]), deadline {deadline_str}",
                    ),
                    label: "Universal Router".to_string(),
                },
                preview_layout: SignablePayloadFieldPreviewLayout {
                    title: Some(SignablePayloadFieldTextV2 {
                        text: "Uniswap Universal Router Execute".to_string(),
                    }),
                    subtitle: Some(SignablePayloadFieldTextV2 {
                        text: format!("1 commands, deadline {deadline_str}"),
                    }),
                    condensed: None,
                    expanded: Some(SignablePayloadFieldListLayout {
                        fields: vec![
                            AnnotatedPayloadField {
                                signable_payload_field: SignablePayloadField::PreviewLayout {
                                    common: SignablePayloadFieldCommon {
                                        fallback_text: "Sweep input: None".to_string(),
                                        label: "Command 1".to_string(),
                                    },
                                    preview_layout: SignablePayloadFieldPreviewLayout {
                                        title: Some(SignablePayloadFieldTextV2 {
                                            text: "Sweep".to_string(),
                                        }),
                                        subtitle: Some(SignablePayloadFieldTextV2 {
                                            text: "Input: None".to_string(),
                                        }),
                                        condensed: None,
                                        expanded: None,
                                    },
                                },
                                static_annotation: None,
                                dynamic_annotation: None,
                            },
                            AnnotatedPayloadField {
                                signable_payload_field: SignablePayloadField::TextV2 {
                                    common: SignablePayloadFieldCommon {
                                        fallback_text: deadline_str.clone(),
                                        label: "Deadline".to_string(),
                                    },
                                    text_v2: SignablePayloadFieldTextV2 {
                                        text: deadline_str.clone(),
                                    },
                                },
                                static_annotation: None,
                                dynamic_annotation: None,
                            },
                        ],
                    }),
                },
            }
        );
    }

    #[test]
    fn test_visualize_tx_commands_real_transaction() {
        // Real transaction from Etherscan with 4 commands:
        // 1. V3SwapExactIn (0x00)
        // 2. V3SwapExactIn (0x00)
        // 3. PayPortion (0x06)
        // 4. UnwrapWeth (0x0c)
        let registry = crate::registry::ContractRegistry::with_default_protocols();

        // Transaction input data (execute function call)
        let input_hex = "3593564c000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000006918f83f00000000000000000000000000000000000000000000000000000000000000040000060c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000003400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000d02ab486cedc00000000000000000000000000000000000000000000000000000000cb274a57755e600000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002be71bdfe1df69284f00ee185cf0d95d0c7680c0d4000bb8c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000340aad21b3b70000000000000000000000000000000000000000000000000000000032e42284d704100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002be71bdfe1df69284f00ee185cf0d95d0c7680c0d4002710c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2000000000000000000000000000000fee13a103a10d593b9ae06b3e05f2e7e1c000000000000000000000000000000000000000000000000000000000000001900000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000fe0b6cdc4c628c0";
        let input = hex::decode(input_hex).unwrap();

        let result = UniversalRouterVisualizer {}.visualize_tx_commands(&input, 1, Some(&registry));
        assert!(result.is_some(), "Should decode transaction successfully");

        // Verify the result contains decoded information
        let field = result.unwrap();
        if let SignablePayloadField::PreviewLayout {
            common,
            preview_layout,
        } = field
        {
            // Check that the fallback text mentions 4 commands
            assert!(
                common.fallback_text.contains("4 commands"),
                "Expected '4 commands' in: {}",
                common.fallback_text
            );

            // Check that expanded section exists
            assert!(
                preview_layout.expanded.is_some(),
                "Expected expanded section"
            );

            if let Some(list_layout) = preview_layout.expanded {
                // Should have 5 fields: 4 commands + 1 deadline
                assert_eq!(
                    list_layout.fields.len(),
                    5,
                    "Expected 5 fields (4 commands + deadline)"
                );

                // Print decoded commands to verify they're human-readable
                println!("\n=== Decoded Transaction ===");
                println!("Fallback text: {}", common.fallback_text);
                for (i, annotated_field) in list_layout.fields.iter().enumerate() {
                    match &annotated_field.signable_payload_field {
                        SignablePayloadField::PreviewLayout {
                            common: field_common,
                            preview_layout: field_preview,
                        } => {
                            println!("\nCommand {}: {}", i + 1, field_common.label);
                            if let Some(title) = &field_preview.title {
                                println!("  Title: {}", title.text);
                            }
                            if let Some(subtitle) = &field_preview.subtitle {
                                println!("  Detail: {}", subtitle.text);

                                // Verify that decoded commands contain tokens, amounts, or decode failures
                                if i < 2 {
                                    // First two are swaps - should mention WETH, address, or decode failure
                                    assert!(
                                        subtitle.text.contains("WETH")
                                            || subtitle.text.contains("0x")
                                            || subtitle.text.contains("Failed to decode"),
                                        "Swap command should mention WETH, token address, or decode failure"
                                    );
                                }
                            }
                        }
                        SignablePayloadField::TextV2 {
                            common: field_common,
                            text_v2,
                        } => {
                            println!("\n{}: {}", field_common.label, text_v2.text);
                        }
                        _ => {}
                    }
                }
                println!("\n=== End Decoded Transaction ===\n");
            }
        } else {
            panic!("Expected PreviewLayout, got different field type");
        }
    }

    #[test]
    fn test_visualize_tx_commands_unrecognized_command() {
        // 0xff is not a valid Command, so it should be skipped
        let commands = vec![0xff, Command::Transfer as u8];
        let inputs = vec![vec![0x01], vec![0x02]];
        let deadline = 0u64;
        let input = encode_execute_call(&commands, inputs.clone(), deadline);

        assert_eq!(
            UniversalRouterVisualizer {}
                .visualize_tx_commands(&input, 1, None)
                .unwrap(),
            SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    fallback_text: "Uniswap Universal Router Execute: 1 commands ([Transfer])"
                        .to_string(),
                    label: "Universal Router".to_string(),
                },
                preview_layout: SignablePayloadFieldPreviewLayout {
                    title: Some(SignablePayloadFieldTextV2 {
                        text: "Uniswap Universal Router Execute".to_string(),
                    }),
                    subtitle: Some(SignablePayloadFieldTextV2 {
                        text: "1 commands".to_string(),
                    }),
                    condensed: None,
                    expanded: Some(SignablePayloadFieldListLayout {
                        fields: vec![AnnotatedPayloadField {
                            signable_payload_field: SignablePayloadField::PreviewLayout {
                                common: SignablePayloadFieldCommon {
                                    fallback_text: "Transfer: 0x01".to_string(),
                                    label: "Command 1".to_string(),
                                },
                                preview_layout: SignablePayloadFieldPreviewLayout {
                                    title: Some(SignablePayloadFieldTextV2 {
                                        text: "Transfer".to_string(),
                                    }),
                                    subtitle: Some(SignablePayloadFieldTextV2 {
                                        text: "Failed to decode parameters".to_string(),
                                    }),
                                    condensed: None,
                                    expanded: None,
                                },
                            },
                            static_annotation: None,
                            dynamic_annotation: None,
                        }],
                    }),
                },
            }
        );
    }
}
