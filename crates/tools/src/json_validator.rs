use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

pub struct JsonValidator;

#[async_trait]
impl Tool for JsonValidator {
    fn id(&self) -> &'static str { "json-validate" }
    fn name(&self) -> &'static str { "JSON 校验" }
    fn description(&self) -> &'static str { "检查 JSON 字符串是否合法，并提供详细的错误信息" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "输入 JSON",
                    "description": "要校验的 JSON 字符串"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Json }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        match serde_json::from_str::<Value>(input_str) {
            Ok(parsed) => {
                // 计算一些统计信息
                let size = input_str.len();
                let depth = json_depth(&parsed);
                Ok(json!({
                    "valid": true,
                    "size": size,
                    "depth": depth,
                    "type": json_type_name(&parsed)
                }))
            }
            Err(e) => {
                Ok(json!({
                    "valid": false,
                    "error": e.to_string(),
                    "line": e.line(),
                    "column": e.column()
                }))
            }
        }
    }
}

fn json_depth(value: &Value) -> u32 {
    match value {
        Value::Object(map) => 1 + map.values().map(json_depth).max().unwrap_or(0),
        Value::Array(arr) => 1 + arr.iter().map(json_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
