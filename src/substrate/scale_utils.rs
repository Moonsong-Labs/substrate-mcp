use scale_value::{Composite, Primitive, Value, ValueDef, Variant};
use serde_json::json;

/// Convert a scale_value::Composite to a serde_json::Value
pub fn composite_to_json<T>(composite: &Composite<T>) -> serde_json::Value {
    match composite {
        Composite::Named(fields) => {
            let mut map = serde_json::Map::new();
            for (name, value) in fields {
                map.insert(name.clone(), value_to_json(value));
            }
            serde_json::Value::Object(map)
        }
        Composite::Unnamed(values) => {
            let array: Vec<_> = values.iter().map(value_to_json).collect();
            serde_json::Value::Array(array)
        }
    }
}

/// Convert a scale_value::Value to a serde_json::Value
pub fn value_to_json<T>(value: &Value<T>) -> serde_json::Value {
    match &value.value {
        ValueDef::Composite(composite) => composite_to_json(composite),
        ValueDef::Variant(variant) => variant_to_json(variant),
        ValueDef::Primitive(primitive) => primitive_to_json(primitive),
        ValueDef::BitSequence(bits) => {
            // Convert bit sequence to hex string using SCALE encoding
            use codec::Encode;
            let encoded = bits.encode();
            json!({
                "bit_sequence": format!("0x{}", hex::encode(&encoded))
            })
        }
    }
}

/// Convert a scale_value::Variant to a serde_json::Value
fn variant_to_json<T>(variant: &Variant<T>) -> serde_json::Value {
    json!({
        "variant": variant.name.clone(),
        "fields": composite_to_json(&variant.values)
    })
}

/// Convert a scale_value::Primitive to a serde_json::Value
fn primitive_to_json(primitive: &Primitive) -> serde_json::Value {
    match primitive {
        Primitive::Bool(b) => json!(b),
        Primitive::Char(c) => json!(c.to_string()),
        Primitive::String(s) => json!(s),
        Primitive::U128(n) => json!(n.to_string()), // Use string for large numbers
        Primitive::U256(n) => {
            // U256 is [u8; 32], convert to hex string
            json!(format!("0x{}", hex::encode(n)))
        }
        Primitive::I128(n) => json!(n.to_string()), // Use string for large numbers
        Primitive::I256(n) => {
            // I256 is [u8; 32], convert to hex string
            json!(format!("0x{}", hex::encode(n)))
        }
    }
}
