use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

pub struct JsonFormatter;

#[async_trait]
impl Tool for JsonFormatter {
    fn id(&self) -> &'static str { "json-format" }
    fn name(&self) -> &'static str { "JSON 格式化" }
    fn description(&self) -> &'static str { "将 JSON 字符串格式化为缩进良好的可读形式" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "输入 JSON",
                    "description": "要格式化的 JSON 字符串"
                },
                "indent": {
                    "type": "integer",
                    "title": "缩进空格数",
                    "default": 2,
                    "minimum": 0,
                    "maximum": 8
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Json }
    fn is_cpu_intensive(&self) -> bool { true }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        let indent = input
            .get("indent")
            .and_then(|v| v.as_i64())
            .unwrap_or(2)
            .max(0)
            .min(8) as usize;

        let parsed: Value = serde_json::from_str(input_str)
            .map_err(|e| ToolError::InvalidInput(format!("JSON 解析失败: {}", e)))?;

        let formatted = serde_json::to_string_pretty(&parsed)
            .map_err(|e| ToolError::ExecutionError(format!("序列化失败: {}", e)))?;

        // 自定义缩进
        let formatted = if indent != 2 {
            let _indent_str = " ".repeat(indent);
            // 用自定义缩进替换默认的 2 空格缩进
            let lines: Vec<String> = formatted.lines().map(|line| {
                let stripped = line.trim_start_matches("  ");
                let leading_spaces = line.len() - stripped.len();
                let new_indent = " ".repeat(leading_spaces / 2 * indent);
                format!("{}{}", new_indent, stripped)
            }).collect();
            lines.join("\n")
        } else {
            formatted
        };

        Ok(json!({
            "result": formatted
        }))
    }
}
