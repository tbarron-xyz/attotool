use serde_yaml::{Mapping, Value as YamlValue};

pub fn parse_and_normalize_yaml(
    input: &str,
    verbose: bool,
) -> Result<Mapping, Box<dyn std::error::Error>> {
    if let Ok(value) = serde_yaml::from_str::<YamlValue>(input) {
        if let YamlValue::Mapping(mapping) = value {
            if mapping.len() > 1 {
                if verbose {
                    println!(
                        "Removed {} additional tool(s) from multi-tool response",
                        mapping.len() - 1
                    );
                }
                let mut new_mapping = Mapping::new();
                if let Some((key, val)) = mapping.iter().next() {
                    new_mapping.insert(key.clone(), val.clone());
                }
                return Ok(new_mapping);
            } else {
                return Ok(mapping);
            }
        }
    }
    Err("Invalid YAML".into())
}

pub fn parse_tool_response_yaml(
    trimmed: &str,
    verbose: bool,
) -> Result<Mapping, Box<dyn std::error::Error>> {
    // First, try parsing the entire trimmed response as YAML
    if let Ok(normalized) = parse_and_normalize_yaml(trimmed, verbose) {
        return Ok(normalized);
    }
    // If parsing the whole failed, try splitting by \n\n and parse the first part
    let yaml_candidate = if let Some(pos) = trimmed.find("\n\n") {
        &trimmed[..pos]
    } else {
        trimmed
    };
    parse_and_normalize_yaml(yaml_candidate, verbose)
}
