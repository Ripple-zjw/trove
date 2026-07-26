use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

/// 简单的哈希计算（不依赖外部哈希库，用标准库实现）
fn compute_md5_like(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes());
    hash.to_hex().to_string()
}

pub struct HashTool;

#[async_trait]
impl Tool for HashTool {
    fn id(&self) -> &'static str { "hash" }
    fn name(&self) -> &'static str { "Hash 计算" }
    fn description(&self) -> &'static str { "计算字符串的 BLAKE3 哈希值" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "输入文本",
                    "description": "要计算哈希的文本"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Crypto }
    fn is_cpu_intensive(&self) -> bool { true }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        let hash = compute_md5_like(input_str);

        Ok(json!({
            "result": hash,
            "algorithm": "BLAKE3",
            "input_length": input_str.len()
        }))
    }
}
