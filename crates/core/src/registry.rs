use std::sync::Arc;

use crate::error::ToolError;
use crate::tool::{Tool, ToolMetadata};

/// 工具注册表
///
/// 使用 Vec 存储所有工具，在工具数 < 200 时 O(n) 查找完全够用。
/// 如果需要极致性能，后期可替换为 phf::Map 或 enum dispatch。
///
/// 使用 Arc 存储工具，以便在需要时（如 spawn 任务）可以 clone Arc 传递所有权。
#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<(String, Arc<dyn Tool>)>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// 注册一个工具
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> &mut Self {
        let id = tool.id().to_string();
        self.tools.push((id, Arc::new(tool)));
        self
    }

    /// 批量注册工具
    pub fn register_all(&mut self, tools: Vec<Arc<dyn Tool>>) -> &mut Self {
        for tool in tools {
            let id = tool.id().to_string();
            self.tools.push((id, tool));
        }
        self
    }

    /// 按 ID 获取工具（返回 Arc 以便跨任务传递）
    pub fn get_arc(&self, id: &str) -> Result<Arc<dyn Tool>, ToolError> {
        self.tools
            .iter()
            .find(|(tid, _)| tid == id)
            .map(|(_, tool)| tool.clone())
            .ok_or_else(|| ToolError::NotFound(id.to_string()))
    }

    /// 按 ID 查找工具（返回引用）
    pub fn get(&self, id: &str) -> Result<&dyn Tool, ToolError> {
        self.tools
            .iter()
            .find(|(tid, _)| tid == id)
            .map(|(_, tool)| tool.as_ref())
            .ok_or_else(|| ToolError::NotFound(id.to_string()))
    }

    /// 获取所有工具的元数据
    pub fn list_metadata(&self) -> Vec<ToolMetadata> {
        let mut metadata: Vec<_> = self.tools.iter().map(|(_, t)| t.metadata()).collect();
        metadata.sort_by(|a, b| {
            a.category
                .order()
                .cmp(&b.category.order())
                .then_with(|| a.id.cmp(b.id))
        });
        metadata
    }

    /// 注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// 工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }
}
