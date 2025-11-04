use async_openai::types::{ResponseFormat, ResponseFormatJsonSchema};
use serde_json::{Map, Value};
use serde_yaml::Mapping;

#[derive(Clone, Debug)]
pub enum ToolResponseFormat {
    Yaml,
    JsonVariableKeys,
    JsonFixedKeys,
}

impl ToolResponseFormat {
    pub fn default() -> Self {
        ToolResponseFormat::Yaml
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "yaml" => Ok(ToolResponseFormat::Yaml),
            "json" => Ok(ToolResponseFormat::JsonVariableKeys),
            "json_fixed_key" => Ok(ToolResponseFormat::JsonFixedKeys),
            _ => Err(format!(
                "Invalid format: {}. Valid options: yaml, json, json_fixed_key",
                s
            )),
        }
    }
}

pub trait ToolResponseParser {
    fn parse(
        &self,
        input: &str,
        verbose: bool,
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error>>;
}

fn convert_yaml_to_json(
    yaml_value: &serde_yaml::Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    match yaml_value {
        serde_yaml::Value::Null => Ok(Value::Null),
        serde_yaml::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Number(serde_json::Number::from(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(
                    serde_json::Number::from_f64(f).ok_or("Invalid float")?,
                ))
            } else {
                Err("Unsupported number type".into())
            }
        }
        serde_yaml::Value::String(s) => Ok(Value::String(s.clone())),
        serde_yaml::Value::Sequence(seq) => {
            let mut arr = Vec::new();
            for item in seq {
                arr.push(convert_yaml_to_json(item)?);
            }
            Ok(Value::Array(arr))
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = Map::new();
            for (k, v) in map {
                if let serde_yaml::Value::String(key_str) = k {
                    obj.insert(key_str.clone(), convert_yaml_to_json(v)?);
                } else {
                    return Err("YAML key is not a string".into());
                }
            }
            Ok(Value::Object(obj))
        }
        serde_yaml::Value::Tagged(_) => {
            Err("Tagged YAML values are not supported".into())
        }
    }
}

fn yaml_mapping_to_json_map(
    mapping: &Mapping,
) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    let mut map = Map::new();
    for (key, val) in mapping {
        if let serde_yaml::Value::String(key_str) = key {
            let json_val = convert_yaml_to_json(val)?;
            map.insert(key_str.clone(), json_val);
        } else {
            return Err("YAML key is not a string".into());
        }
    }
    Ok(map)
}

struct YamlParser;

impl ToolResponseParser for YamlParser {
    fn parse(
        &self,
        input: &str,
        verbose: bool,
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(input) {
            if let serde_yaml::Value::Mapping(mapping) = value {
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
                    return yaml_mapping_to_json_map(&new_mapping);
                } else {
                    return yaml_mapping_to_json_map(&mapping);
                }
            }
        }
        Err("Invalid YAML".into())
    }
}

struct JsonVariableKeysParser;

impl ToolResponseParser for JsonVariableKeysParser {
    fn parse(
        &self,
        input: &str,
        verbose: bool,
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        if let Ok(value) = serde_json::from_str::<Value>(input) {
            if let Value::Object(map) = value {
                if map.len() > 1 {
                    if verbose {
                        println!(
                            "Removed {} additional tool(s) from multi-tool response",
                            map.len() - 1
                        );
                    }
                    let mut new_map = Map::new();
                    if let Some((key, val)) = map.iter().next() {
                        new_map.insert(key.clone(), val.clone());
                    }
                    return Ok(new_map);
                } else {
                    return Ok(map);
                }
            }
        }
        Err("Invalid JSON".into())
    }
}

struct JsonFixedKeysParser;

impl ToolResponseParser for JsonFixedKeysParser {
    fn parse(
        &self,
        input: &str,
        verbose: bool,
    ) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
        if let Ok(value) = serde_json::from_str::<Value>(input) {
            if let Value::Object(map) = value {
                if let (Some(Value::String(tool)), Some(args)) =
                    (map.get("tool"), map.get("tool_args"))
                {
                    let mut result = Map::new();
                    result.insert(tool.clone(), args.clone());
                    return Ok(result);
                }
            }
        }
        Err("Invalid JSON".into())
    }
}

pub fn parse_tool_response(
    format: &ToolResponseFormat,
    input: &str,
    verbose: bool,
) -> Result<Map<String, Value>, Box<dyn std::error::Error>> {
    match format {
        ToolResponseFormat::Yaml => YamlParser.parse(input, verbose),
        ToolResponseFormat::JsonVariableKeys => {
            JsonVariableKeysParser.parse(input, verbose)
        }
        ToolResponseFormat::JsonFixedKeys => {
            JsonFixedKeysParser.parse(input, verbose)
        }
    }
}
pub fn response_format(
    tool_response_format: &ToolResponseFormat,
    tool_names: &Vec<serde_json::Value>,
) -> Option<ResponseFormat> {
    return match tool_response_format {
        ToolResponseFormat::JsonFixedKeys => {
            let schema = serde_json::json!({
                "type": "object",
                "properties": {
                    "tool": {
                        "type": "string",
                        "enum": tool_names
                    },
                    "tool_args": {
                        "type": "object",
                        "additionalProperties": {
                            "anyOf": [
                                {"type": "string"},
                                {"type": "number"}
                            ]
                        }
                    }
                },
                "required": ["tool", "tool_args"],
                "additionalProperties": false
            });
            Some(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    name: "tool_call".to_string(),
                    schema: Some(schema),
                    strict: Some(true),
                    description: Some("A single tool call".to_string()),
                },
            })
        }
        _ => None,
    };
}
