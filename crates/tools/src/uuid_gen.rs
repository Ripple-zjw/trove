use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolResult};

pub struct UuidGen;

#[async_trait]
impl Tool for UuidGen {
    fn id(&self) -> &'static str { "uuid-gen" }
    fn name(&self) -> &'static str { "UUID 生成" }
    fn description(&self) -> &'static str { "生成 UUID v4 标识符，支持批量生成" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "count": {
                    "type": "integer",
                    "title": "生成数量",
                    "description": "一次生成的 UUID 数量",
                    "default": 1,
                    "minimum": 1,
                    "maximum": 100
                },
                "uppercase": {
                    "type": "boolean",
                    "title": "大写",
                    "description": "是否输出大写",
                    "default": false
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Utility }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let count = input
            .get("count")
            .and_then(|v| v.as_i64())
            .unwrap_or(1)
            .clamp(1, 100) as usize;

        let uppercase = input
            .get("uppercase")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let uuids: Vec<String> = (0..count)
            .map(|_| {
                let uuid = uuid::Uuid::new_v4().to_string();
                if uppercase {
                    uuid.to_uppercase()
                } else {
                    uuid
                }
            })
            .collect();

        Ok(json!({
            "result": if count == 1 { Value::String(uuids[0].clone()) } else { Value::String(uuids.join("\n")) },
            "uuids": uuids,
            "count": count
        }))
    }
}
