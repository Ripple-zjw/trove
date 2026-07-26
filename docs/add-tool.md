# Trove 添加新工具指南

添加一个新工具只需 3 步：**创建文件 → 实现 Trait → 注册一行**。前端自动根据 `input_schema` 生成表单——无需额外编码。

---

## 步骤一：创建 Rust 文件

在 `crates/tools/src/` 下创建新文件，如 `your_tool.rs`。

## 步骤二：实现 Tool trait

```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use trove_core::{Tool, ToolCategory, ToolContext, ToolError, ToolResult};

pub struct YourTool;

#[async_trait]
impl Tool for YourTool {
    /// 工具 ID（kebab-case，全局唯一，用于 URL/CLI 路由）
    fn id(&self) -> &'static str { "your-tool-id" }

    /// 人类可读的中文名称
    fn name(&self) -> &'static str { "工具名称" }

    /// 一句话描述
    fn description(&self) -> &'static str { "这是做什么的" }

    /// JSON Schema → 前端自动生成表单
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["input"],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "输入",
                    "description": "输入描述"
                },
                // 更多参数...
            }
        })
    }

    /// 工具分类（用于前端分组展示）
    fn category(&self) -> ToolCategory { ToolCategory::Utility }

    /// 是否 CPU 密集型（设为 true 会走 tokio::spawn 隔离）
    fn is_cpu_intensive(&self) -> bool { false }

    /// 执行逻辑
    async fn execute(&self, input: Value, _ctx: ToolContext) -> ToolResult<Value> {
        let input_str = input.get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("缺少 input 字段".to_string()))?;

        // TODO: 业务逻辑
        let result = format!("处理: {}", input_str);

        Ok(json!({ "result": result }))
    }
}
```

### JSON Schema 字段说明

| 字段 | 用途 |
|------|------|
| `title` | 前端表单 label |
| `description` | 帮助提示文本 |
| `default` | 默认值 |
| `enum` | 下拉选择框 |
| `minimum` / `maximum` | 数值范围 |
| `required` | 必填标记 |

### 可选：需要额外 Crate 依赖

在 `crates/tools/Cargo.toml` 的 `[dependencies]` 中添加。

## 步骤三：注册

打开 `crates/tools/src/lib.rs`，在 `register_all()` 中加一行：

```rust
pub fn register_all(registry: &mut trove_core::ToolRegistry) {
    registry
        // ... 已有工具
        .register(your_tool::YourTool);  // ← 新加这行
}
```

## 验证

```bash
# 编译
cargo build

# 列表确认
./target/debug/trove list | grep your-tool-id

# API 测试
curl --noproxy '*' -X POST http://127.0.0.1:8080/api/tools/your-tool-id/execute \
  -H 'Content-Type: application/json' \
  -d '{"input":{"input":"测试"}}'

# CLI 测试
./target/debug/trove exec your-tool-id --input '{"input":"测试"}'
```

## 规则一览

| 项目 | 规则 |
|------|------|
| ID 格式 | 全局唯一 kebab-case（`json-format`、`base64-encode`） |
| 名称 | 中文 |
| 错误 | 统一用 `ToolError::InvalidInput` 或 `ToolError::ExecutionError` |
| 依赖 | 加在 `crates/tools/Cargo.toml` |
| 前端 | 自动生成表单，无需编码 |
| 注册 | 只需一行 `.register(...)` |
