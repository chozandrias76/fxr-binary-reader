pub mod view {
    use crate::fxr::*;
    use serde_reflection::{ContainerFormat, Format, Named, Samples, Tracer, TracerConfig};
    use std::collections::BTreeMap;

    #[derive(serde::Serialize)]
    pub struct HeaderView<'a> {
        id: u32,
        header: &'a Header,
    }

    use log::debug;

    #[derive(Debug, Clone)]
    pub struct StructNode {
        pub name: String,
        pub fields: Vec<(String, String)>, // (field name, type name)
        pub children: Vec<StructNode>,
        pub is_expanded: bool,
    }

    /// Build a reflection tree for a given type
    /// # Arguments
    /// * `sample` - A sample instance of the type to reflect
    /// * `name` - The name of the type to reflect
    /// # Returns
    /// A `StructNode` representing the reflection tree of the type
    /// # Panics
    /// Panics if the type is not found in the registry
    /// # Example
    /// ```rust
    /// use fxr_binary_reader::fxr::view::view::build_reflection_tree;
    /// use fxr_binary_reader::fxr::Header;
    /// use fxr_binary_reader::fxr::view::view::HeaderView;
    /// // Implement the build_reflection_tree function
    /// let header = Header::default();
    ///
    /// let tree = build_reflection_tree(&header, "Header");
    ///
    /// assert_eq!(tree.name, "Header");
    /// assert_eq!(tree.fields.len(), 37, "Expected 37 fields in Header");
    /// assert_eq!(tree.children.len(), 0, "Expected no children in Header");
    /// assert_eq!(tree.is_expanded, false, "Expected tree to not be expanded");
    ///
    /// // Assert on the fields
    /// assert_eq!(tree.fields[0].0, "magic", "Expected first field to be 'magic'");
    /// assert_eq!(tree.fields[0].1, "TupleArray { content: U8, size: 4 , value: Some(Array [Number(70), Number(88), Number(82), Number(0)]) }", "TupleArray {{ content: U8, size: 4 , value: Some(Array [Number(70), Number(88), Number(82), Number(0)]) }}");
    /// assert_eq!(tree.fields[36].0, "unk8c", "Expected last field to be 'unk8c'");
    /// ```
    pub fn build_reflection_tree<T: serde::Serialize + ?Sized>(
        sample: &T,
        name: &str,
    ) -> StructNode {
        let config = TracerConfig::default();
        let mut tracer = Tracer::new(config);
        let mut samples = Samples::new();

        tracer.trace_value(&mut samples, sample).unwrap();
        let registry: BTreeMap<String, ContainerFormat> = tracer.registry().unwrap();

        let struct_desc: &ContainerFormat = registry
            .get(name)
            .unwrap_or_else(|| panic!("Type not found in registry: {}", name));

        let mut fields = vec![];
        let mut children = vec![];

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

                fields.push((field_name.clone(), formatted_value));

                // Optional: resolve sub-structs from registry
                if let Format::TypeName(child_type_name) = field_type {
                    if registry.get(child_type_name).is_some() {
                        children.push(build_reflection_tree(&(), child_type_name));
                    }
                }
            }
        } else {
            panic!("Expected a struct format");
        }
        let debug = format!("{:#?}", struct_desc);
        debug!("Debug StructNode: {}", debug);
        StructNode {
            name: name.to_string(),
            fields,
            children,
            is_expanded: false,
        }
    }
}
