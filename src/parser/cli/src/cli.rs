use crate::chains;
use chains::{available_chains, parse_chain};
use clap::{Arg, Command};
use generated::parser::{EthereumMetadata, Abi, Idl, SolanaMetadata, SolanaIdlType, parse_request::ChainMetadata};
use parser_app::registry::create_registry;
use visualsign::vsptrait::VisualSignOptions;

fn parse_and_display(chain: &str, raw_tx: &str, options: VisualSignOptions, output_format: &str) {
    let registry_chain = parse_chain(chain);

    let registry = create_registry();
    let signable_payload_str = registry.convert_transaction(&registry_chain, raw_tx, options);
    match signable_payload_str {
        Ok(payload) => match output_format {
            "json" => {
                if let Ok(json_output) = serde_json::to_string_pretty(&payload) {
                    println!("{json_output}");
                } else {
                    eprintln!("Error: Failed to serialize output as JSON");
                }
            }
            "text" => {
                println!("{payload:#?}");
            }
            _ => {
                eprintln!("Error: Unsupported output format '{output_format}'");
            }
        },
        Err(err) => {
            eprintln!("Error: {err:?}");
        }
    }
}

/// app cli
pub struct Cli;
impl Cli {
    /// start the parser cli
    ///
    /// # Panics
    ///
    /// Executes the CLI application, parsing command line arguments and processing the transaction
    pub fn execute() {
        let chains = available_chains();
        let chain_help = format!("Chain type ({})", chains.join(", "));

        let matches = Command::new("visualsign-parser")
            .version("1.0")
            .about("Converts raw transactions to visual signing properties")
            .arg(
                Arg::new("chain")
                    .short('c')
                    .long("chain")
                    .value_name("CHAIN")
                    .help(&chain_help)
                    .value_parser(chains.clone())
                    .required(true),
            )
            .arg(
                Arg::new("transaction")
                    .short('t')
                    .long("transaction")
                    .value_name("RAW_TX")
                    .help("Raw transaction hex string")
                    .required(true),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FORMAT")
                    .help("Output format")
                    .value_parser(["text", "json"])
                    .default_value("text"),
            )
            .arg(
                Arg::new("abi")
                    .long("abi")
                    .value_name("ABI_JSON")
                    .help("(Ethereum) Contract ABI as a JSON string")
                    .required(false),
            )
            .arg(
                Arg::new("idl")
                    .long("idl")
                    .value_name("IDL_JSON")
                    .help("(Solana) Program IDL as a JSON string (default: Anchor)")
                    .required(false),
            )
            .arg(
                Arg::new("idl-type")
                    .long("idl-type")
                    .value_name("IDL_TYPE")
                    .help("(Solana) IDL type (default: anchor)")
                    .value_parser(["anchor"])
                    .default_value("anchor")
                    .required(false),
            )
            .get_matches();

        let chain = matches
            .get_one::<String>("chain")
            .expect("Chain is required");
        let raw_tx = matches
            .get_one::<String>("transaction")
            .expect("Transaction is required");
        let output_format = matches
            .get_one::<String>("output")
            .expect("Output format has default value");
        let abi = matches.get_one::<String>("abi").cloned();
        let idl = matches.get_one::<String>("idl").cloned();
        let idl_type = matches
            .get_one::<String>("idl-type")
            .expect("IDL type has default value");

        let options = match chain.as_str() {
            "ethereum" => {
                VisualSignOptions {
                    decode_transfers: true,
                    transaction_name: None,
                    metadata: abi.map(|abi_str| {
                        ChainMetadata::Ethereum(EthereumMetadata {
                            abi: Some(Abi {
                                value: abi_str,
                                signature: None,
                            }),
                        })
                    }),
                }
            }
            "solana" => {
                VisualSignOptions {
                    decode_transfers: true,
                    transaction_name: None,
                    metadata: idl.map(|idl_str| {
                        let idl_type_enum = match idl_type.as_str() {
                            "anchor" | _ => SolanaIdlType::Anchor,
                        };
                        ChainMetadata::Solana(SolanaMetadata {
                            idl: Some(Idl {
                                value: idl_str,
                                idl_type: idl_type_enum as i32,
                                idl_version: None,
                                signature: None,
                            }),
                        })
                    }),
                }
            }
            _ => {
                VisualSignOptions {
                    decode_transfers: true,
                    transaction_name: None,
                    metadata: None,
                }
            }
        };

        parse_and_display(chain, raw_tx, options, output_format);
    }
}
