use async_trait::async_trait;
use base64::Engine;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

pub struct Base64Encode;

#[async_trait]
impl Tool for Base64Encode {
    fn id(&self) -> &'static str { "base64-encode" }
    fn name(&self) -> &'static str { "Base64 编码" }
    fn description(&self) -> &'static str { "将字符串或二进制数据编码为 Base64" }

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

    fn category(&self) -> ToolCategory { ToolCategory::Crypto }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        let encoded = base64::engine::general_purpose::STANDARD.encode(input_str);
        Ok(json!({ "result": encoded }))
    }
}

pub struct Base64Decode;

#[async_trait]
impl Tool for Base64Decode {
    fn id(&self) -> &'static str { "base64-decode" }
    fn name(&self) -> &'static str { "Base64 解码" }
    fn description(&self) -> &'static str { "将 Base64 编码的字符串解码为原始文本" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "Base64 字符串",
                    "description": "要解码的 Base64 字符串"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Crypto }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        let engine = base64::engine::general_purpose::STANDARD;
        match engine.decode(input_str) {
            Ok(bytes) => {
                match String::from_utf8(bytes.clone()) {
                    Ok(text) => Ok(json!({ "result": text })),
                    Err(_) => {
                        let hex_str: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                        Ok(json!({ "result_hex": hex_str }))
                    }
                }
            }
            Err(e) => Err(ToolError::InvalidInput(format!("Base64 解码失败: {}", e))),
        }
    }
}
