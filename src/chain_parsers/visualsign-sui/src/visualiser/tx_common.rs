use crate::visualiser;
use crate::visualiser::helper_field::{
    create_simple_text_field, create_address_field, create_amount_field, create_raw_data_field, create_text_field,
};

use sui_json_rpc_types::{SuiTransactionBlockData, SuiTransactionBlockDataAPI};
use sui_types::transaction::TransactionData;

use visualsign::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon,
    SignablePayloadFieldListLayout,
};

pub fn add_tx_network(fields: &mut Vec<SignablePayloadField>) {
    fields.push(create_simple_text_field("Network", "Sui Network"));
}

pub fn add_tx_details(
    fields: &mut Vec<SignablePayloadField>,
    tx_data: &TransactionData,
    block_data: &SuiTransactionBlockData,
) {
    let mut payload_fields: Vec<AnnotatedPayloadField> = vec![];

    payload_fields.extend(create_tx_type_fields(block_data));
    payload_fields.extend(create_tx_gas_fields(block_data));
    payload_fields.extend(create_tx_data_fields(tx_data));

    fields.push(SignablePayloadField::ListLayout {
        common: SignablePayloadFieldCommon {
            fallback_text: "Transaction Details".to_string(),
            label: "Transaction Details".to_string(),
        },
        list_layout: SignablePayloadFieldListLayout {
            fields: payload_fields,
        },
    });
}

fn create_tx_type_fields(block_data: &SuiTransactionBlockData) -> Vec<AnnotatedPayloadField> {
    vec![create_text_field(
        "Transaction Type",
        &visualiser::determine_transaction_type_string(block_data),
    )]
}

fn create_tx_gas_fields(block_data: &SuiTransactionBlockData) -> Vec<AnnotatedPayloadField> {
    vec![
        create_address_field(
            "Gas Owner",
            &visualiser::helper_address::truncate_address(&block_data.gas_data().owner.to_string()),
            None,
            None,
            None,
            None,
        ),
        create_amount_field(
            "Gas Budget",
            &block_data.gas_data().budget.to_string(),
            "MIST",
        ),
        create_amount_field(
            "Gas Price",
            &block_data.gas_data().price.to_string(),
            "MIST",
        ),
    ]
}

fn create_tx_data_fields(tx_data: &TransactionData) -> Vec<AnnotatedPayloadField> {
    if let Ok(encoded) = bcs::to_bytes::<TransactionData>(tx_data) {
        vec![create_raw_data_field(&encoded)]
    } else {
        vec![]
    }
}
