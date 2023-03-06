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
  payment_method_name: String,
  cart_total: f64
}

// Parse the JSON metafield value using serde
impl Configuration {
  fn from_str(value: &str) -> Self {
    serde_json::from_str(value).expect("Unable to parse configuration value from metafield")
  }
}

// Use the shopify_function crate to declare your function entrypoint
#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
  let no_changes = output::FunctionResult { operations: vec![] };

  // Get the configuration from the metafield on your function owner
  let config = match input.payment_customization.metafield {
    Some(input::InputPaymentCustomizationMetafield { value }) => Configuration::from_str(&value),
    None => return Ok(no_changes),
  };

  // Use the configured cart total instead of a hardcoded value
  let cart_total = input.cart.cost.total_amount.amount.parse::<f64>().unwrap();
  if cart_total < config.cart_total {
    eprintln!("Cart total is not high enough, no need to hide the payment method.");
    return Ok(no_changes);
  }

  // Use the configured payment method name instead of a hardcoded value
  let hide_payment_method = input.payment_methods
    .iter()
    .find(|&method| method.name.contains(&config.payment_method_name.to_string()))
    .map(|method| output::HideOperation {
        payment_method_id: method.id.to_string()
    });

  Ok(output::FunctionResult { operations: vec![
      output::Operation {
        hide: hide_payment_method,
        move_: None,
        rename: None
      }
  ] })
}

#[cfg(test)]
mod tests;
