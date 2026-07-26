use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

use std::collections::HashMap;

pub struct TextStats;

#[async_trait]
impl Tool for TextStats {
    fn id(&self) -> &'static str { "text-stats" }
    fn name(&self) -> &'static str { "文本统计" }
    fn description(&self) -> &'static str { "分析文本的字符数、单词数、行数、字频等统计信息" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "输入文本",
                    "description": "要分析的文本"
                },
                "show_word_freq": {
                    "type": "boolean",
                    "title": "显示词频",
                    "description": "是否显示词频统计",
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

        let show_freq = input
            .get("show_word_freq")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let char_count = input_str.chars().count();
        let char_no_space: usize = input_str.chars().filter(|c| !c.is_whitespace()).count();
        let word_count = input_str.split_whitespace().count();
        let line_count = if input_str.is_empty() { 0 } else { input_str.lines().count() };
        let byte_count = input_str.len();

        // 英文字母/数字/标点统计
        let letter_count = input_str.chars().filter(|c| c.is_ascii_alphabetic()).count();
        let digit_count = input_str.chars().filter(|c| c.is_ascii_digit()).count();
        let space_count = input_str.chars().filter(|c| c.is_whitespace()).count();
        let punctuation_count = input_str.chars().filter(|c| c.is_ascii_punctuation()).count();
        let cjk_count = input_str.chars().filter(|c| {
            matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}')
        }).count();

        let mut result = json!({
            "char_count": char_count,
            "char_count_no_space": char_no_space,
            "word_count": word_count,
            "line_count": line_count,
            "byte_count": byte_count,
            "letter_count": letter_count,
            "digit_count": digit_count,
            "space_count": space_count,
            "punctuation_count": punctuation_count,
            "cjk_char_count": cjk_count,
        });

        if show_freq && word_count > 0 {
            let mut freq: HashMap<&str, usize> = HashMap::new();
            for word in input_str.split_whitespace() {
                let word = word.trim_matches(|c: char| c.is_ascii_punctuation());
                if !word.is_empty() {
                    *freq.entry(word).or_default() += 1;
                }
            }
            let mut freq_vec: Vec<_> = freq.into_iter().collect();
            freq_vec.sort_by_key(|k| std::cmp::Reverse(k.1));
            let top_words: Vec<Value> = freq_vec.iter().take(50).map(|(w, c)| {
                json!({ "word": w, "count": c })
            }).collect();

            result["word_frequency"] = json!(top_words);
        }

        Ok(result)
    }
}
