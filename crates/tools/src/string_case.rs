use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

pub struct StringCase;

#[async_trait]
impl Tool for StringCase {
    fn id(&self) -> &'static str { "string-case" }
    fn name(&self) -> &'static str { "字符串格式转换" }
    fn description(&self) -> &'static str { "字符串大小写转换、驼峰/蛇形/帕斯卡命名格式互转" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "输入文本",
                    "description": "要转换的字符串"
                },
                "to_case": {
                    "type": "string",
                    "title": "目标格式",
                    "description": "转换格式",
                    "enum": ["lowercase", "uppercase", "capitalize", "camelCase", "snake_case", "PascalCase", "kebab-case", "CONSTANT_CASE"],
                    "default": "lowercase"
                },
                "reverse": {
                    "type": "boolean",
                    "title": "反转",
                    "description": "是否反转字符串",
                    "default": false
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::Text }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        let to_case = input
            .get("to_case")
            .and_then(|v| v.as_str())
            .unwrap_or("lowercase");

        let should_reverse = input
            .get("reverse")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let reversed = if should_reverse {
            input_str.chars().rev().collect::<String>()
        } else {
            input_str.to_string()
        };

        let result = match to_case {
            "lowercase" => reversed.to_lowercase(),
            "uppercase" => reversed.to_uppercase(),
            "capitalize" => {
                let mut c = reversed.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().to_string() + c.as_str(),
                }
            }
            "camelCase" => to_camel_case(&reversed),
            "snake_case" => to_snake_case(&reversed),
            "PascalCase" => to_pascal_case(&reversed),
            "kebab-case" => to_kebab_case(&reversed),
            "CONSTANT_CASE" => to_constant_case(&reversed),
            _ => reversed,
        };

        Ok(json!({
            "result": result,
            "original": input_str,
            "to_case": to_case
        }))
    }
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    let camel = to_camel_case(s);
    let mut c = camel.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

fn to_constant_case(s: &str) -> String {
    let snake = to_snake_case(s);
    snake.to_uppercase()
}
