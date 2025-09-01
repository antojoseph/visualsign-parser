use crate::registry::{CommandVisualizer, VisualizerContext, decode_calldata};
use visualsign::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon,
    SignablePayloadFieldListLayout,
};

pub struct Eip7730TxVisualizer;

impl CommandVisualizer for Eip7730TxVisualizer {
    fn visualize_tx_commands(&self, context: &VisualizerContext) -> Option<SignablePayloadField> {
        let decoded = decode_calldata(context.calldata)?;
        if decoded.is_empty() {
            return None;
        }

        // Create one item per decoded field, using the actual field types and values
        let items: Vec<AnnotatedPayloadField> = decoded
            .into_iter()
            .map(|field| AnnotatedPayloadField {
                signable_payload_field: field,
                static_annotation: None,
                dynamic_annotation: None,
            })
            .collect();

        let summary_text = format!("Decoded {} field(s)", items.len());

        Some(SignablePayloadField::ListLayout {
            common: SignablePayloadFieldCommon {
                fallback_text: summary_text,
                label: "Decoded Input".to_string(),
            },
            list_layout: SignablePayloadFieldListLayout { fields: items },
        })
    }
}
