use shopify_function::prelude::*;
use shopify_function::Result;

use serde::{Deserialize, Serialize};

// Use the shopify_function crate to generate structs for the function input and output
generate_types!(
    query_path = "./input.graphql",
    schema_path = "./schema.graphql"
);

// Create a structure that matches the JSON structure that you'll use for your configuration
#[derive(Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Configuration {
    zip: String,
    message: String
}

// Parse the JSON metafield value using serde
impl Configuration {
    fn from_str(value: &str) -> Self {
        serde_json::from_str(value).expect("Unable to parse configuration value from metafield")
    }
}

#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    let no_changes = output::FunctionResult { operations: vec![] };

    // Get the configuration from the metafield on your function owner
    let config = match input.delivery_customization.metafield {
        Some(input::InputDeliveryCustomizationMetafield { value }) =>
            Configuration::from_str(&value),
        None => return Ok(no_changes),
    };

    let to_rename = input.cart.delivery_groups
        .iter()
        // Filter for delivery groups with a shipping address containing the affected state or province
        .filter(|group| {
            let postal_code = group.delivery_address.as_ref()
                .and_then(|address| address.zip.as_ref());
            match postal_code {
                Some(code) => code == &config.zip,
                None => false
            }
        })
        // Collect the delivery options from these groups
        .flat_map(|group| &group.delivery_options)
        // Construct a rename operation for each, adding the message to the option title
        .map(|option| output::RenameOperation {
            delivery_option_handle: option.handle.to_string(),
            title: match &option.title {
                Some(title) => format!("{} - {}", title, config.message),
                None => config.message.to_string()
            }
        })
        // Wrap with an Operation
        .map(|rename| output::Operation {
            rename: Some(rename),
            hide: None,
            move_: None
        })
        .collect();

    // The shopify_function crate serializes your function result and writes it to STDOUT
    Ok(output::FunctionResult { operations: to_rename })
}

#[cfg(test)]
mod tests;
