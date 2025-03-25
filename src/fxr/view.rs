use log::debug;
use ratatui_tree_widget::TreeItem;
use serde_reflection::{ContainerFormat, Format, Named, Samples, Tracer, TracerConfig};
use std::{collections::BTreeMap, error::Error};

/// Build a reflection tree for a given type
/// # Arguments
/// * `sample` - A sample instance of the type to reflect
/// * `name` - The name of the type to reflect
/// # Returns
/// A `TreeItem` representing the reflection tree of the type
/// # Panics
/// Panics if the type is not found in the registry
pub fn build_reflection_tree<T: serde::Serialize + ?Sized>(
    sample: &T,
    name: &str,
) -> Result<TreeItem<'static>, Box<dyn Error>> {
    let config = TracerConfig::default();
    let mut tracer = Tracer::new(config);
    let mut samples = Samples::new();

    tracer.trace_value(&mut samples, sample)?;
    let registry: BTreeMap<String, ContainerFormat> = tracer.registry()?;

    let struct_desc: &ContainerFormat = registry.get(name).unwrap_or_else(|| {
        panic!(
            "Type not found in registry: {}. Contents: {:#?}",
            name,
            registry
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        )
    });

    let mut children_items = vec![];

    if let ContainerFormat::Struct(fields_vec) = struct_desc {
        for Named {
            name: field_name,
            value: field_type,
        } in fields_vec.iter()
        {
            // Extract the value of the field using serde_json
            let field_value = serde_json::to_value(sample)
                .map(|v| v.get(field_name).cloned())
                .unwrap_or(Some(serde_json::Value::Null));

            // Handle TupleArray formatting
            let formatted_value = if let Format::TupleArray { content, size } = field_type {
                if let Some(serde_json::Value::Array(array)) = field_value {
                    let array_values: Vec<String> =
                        array.iter().map(|v| format!("{:?}", v)).collect();
                    format!(
                        "TupleArray {{ content: {:?}, size: {} , value: Some(Array [{}]) }}",
                        content,
                        size,
                        array_values.join(", ")
                    )
                } else {
                    format!(
                        "TupleArray {{ content: {:?}, size: {} , value: None }}",
                        content, size
                    )
                }
            } else {
                format!("{:?}", field_value)
            };

            // Create a child TreeItem for this field
            let mut field_item = TreeItem::new_leaf(format!("{}: {}", field_name, formatted_value));

            // Optional: resolve sub-structs from registry
            if let Format::TypeName(child_type_name) = field_type {
                if registry.get(child_type_name).is_some() {
                    let child_tree = build_reflection_tree(&(), child_type_name);
                    field_item =
                        TreeItem::new(format!("{}: {}", field_name, formatted_value), Vec::new());
                    field_item.add_child(child_tree?);
                }
            }

            children_items.push(field_item);
        }
    } else {
        panic!("Expected a struct format");
    }

    let debug = format!("{:#?}", struct_desc);
    debug!("Debug TreeItem: {}", debug);

    // Return the root TreeItem
    Ok(TreeItem::new(name.to_string(), children_items))
}
