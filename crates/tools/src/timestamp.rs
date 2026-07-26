use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

pub struct TimestampToDate;

#[async_trait]
impl Tool for TimestampToDate {
    fn id(&self) -> &'static str { "ts-to-date" }
    fn name(&self) -> &'static str { "时间戳 → 日期" }
    fn description(&self) -> &'static str { "将 Unix 时间戳转换为可读的日期时间字符串" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["timestamp"],
            "properties": {
                "timestamp": {
                    "type": "string",
                    "title": "时间戳",
                    "description": "Unix 时间戳（秒或毫秒）"
                },
                "format": {
                    "type": "string",
                    "title": "输出格式",
                    "description": "日期格式，默认 ISO 8601",
                    "default": "%Y-%m-%d %H:%M:%S"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::DateTime }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let ts_str = input
            .get("timestamp")
            .and_then(|v| v.as_str())
            .or_else(|| input.get("timestamp").and_then(|v| v.as_i64()).map(|n| {
                // For numeric timestamps, we'll handle below
                return Box::leak(Box::new(n.to_string())).as_str();
            }))
            .unwrap_or("")
            .to_string();

        let ts: i64 = ts_str.parse()
            .map_err(|_| ToolError::InvalidInput(format!("无效的时间戳: {}", ts_str)))?;

        let format_str = input
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("%Y-%m-%d %H:%M:%S");

        let (secs, nsecs) = if ts > 1_000_000_000_000 {
            // 毫秒时间戳
            (ts / 1000, (ts % 1000) as u32 * 1_000_000)
        } else {
            (ts, 0)
        };

        let datetime = DateTime::from_timestamp(secs, nsecs)
            .ok_or_else(|| ToolError::InvalidInput("时间戳超出范围".to_string()))?;
        let datetime_utc: DateTime<Utc> = datetime; // already UTC for from_timestamp
        let formatted = datetime_utc.format(format_str).to_string();

        Ok(json!({
            "result": formatted,
            "iso_8601": datetime_utc.to_rfc3339(),
            "timestamp_secs": secs,
            "timestamp_ms": ts
        }))
    }
}

pub struct DateToTimestamp;

#[async_trait]
impl Tool for DateToTimestamp {
    fn id(&self) -> &'static str { "date-to-ts" }
    fn name(&self) -> &'static str { "日期 → 时间戳" }
    fn description(&self) -> &'static str { "将日期时间字符串转换为 Unix 时间戳" }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["date"],
            "properties": {
                "date": {
                    "type": "string",
                    "title": "日期时间",
                    "description": "日期时间字符串，如 \"2024-01-01 00:00:00\" 或 \"2024-01-01T00:00:00Z\""
                },
                "format": {
                    "type": "string",
                    "title": "输入格式",
                    "description": "输入日期的格式，默认自动检测",
                    "default": "auto"
                }
            }
        })
    }

    fn category(&self) -> ToolCategory { ToolCategory::DateTime }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let date_str = input
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 date 字段".to_string()))?;

        let datetime = if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            dt.with_timezone(&Utc)
        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
            DateTime::from_naive_utc_and_offset(dt, Utc)
        } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
            DateTime::from_naive_utc_and_offset(dt, Utc)
        } else if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let naive = dt.and_hms_opt(0, 0, 0).unwrap();
            DateTime::from_naive_utc_and_offset(naive, Utc)
        } else {
            return Err(ToolError::InvalidInput(format!("无法解析日期: {}", date_str)));
        };

        let ts_secs = datetime.timestamp();
        let ts_ms = datetime.timestamp_millis();

        Ok(json!({
            "timestamp_secs": ts_secs,
            "timestamp_ms": ts_ms
        }))
    }
}
