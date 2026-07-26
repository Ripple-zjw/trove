use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

use std::fmt::Write;

pub struct UrlEncode;

fn url_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                write!(result, "%{:02X}", byte).unwrap();
            }
        }
    }
    result
}

fn url_decode(input: &str) -> Result<String, ToolError> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() < 2 {
                    return Err(ToolError::InvalidInput("不完整的百分号编码".to_string()));
                }
                let byte = u8::from_str_radix(&hex, 16)
                    .map_err(|_| ToolError::InvalidInput(format!("无效的百分号编码: %{}", hex)))?;
                result.push(byte as char);
            }
            _ => result.push(c),
        }
    }
    Ok(result)
}

#[async_trait]
impl Tool for UrlEncode {
    fn id(&self) -> &'static str { "url-encode" }
    fn name(&self) -> &'static str { "URL 编码" }
    fn description(&self) -> &'static str { "将文本进行 URL 百分号编码（RFC 3986）" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "输入文本",
                    "description": "要编码的文本"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Network }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        let encoded = url_encode(input_str);
        Ok(json!({ "result": encoded }))
    }
}

pub struct UrlDecode;

#[async_trait]
impl Tool for UrlDecode {
    fn id(&self) -> &'static str { "url-decode" }
    fn name(&self) -> &'static str { "URL 解码" }
    fn description(&self) -> &'static str { "解码 URL 百分号编码的字符串" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "编码的 URL 字符串",
                    "description": "要解码的 URL 编码字符串"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Network }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        let decoded = url_decode(input_str)?;
        Ok(json!({ "result": decoded }))
    }
}
