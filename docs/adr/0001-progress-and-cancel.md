# ADR-0001: 长耗时工具的进度推送与取消机制

## 状态

已实现

## 上下文

`video-concat` 工具调用系统 ffmpeg 进行视频拼接，可能需要数分钟。现有 Tool trait 的 execute 方法只返回 `ToolResult<Value>`，没有进度推送和取消机制。

## 决策

在 `ToolContext` 中增加两个可选字段：

1. `progress_tx: Option<mpsc::UnboundedSender<ProgressEvent>>` — 用于推送进度事件
2. `cancel_token: Option<CancelToken>` — 用于取消操作（基于 `Arc<AtomicBool>`）

工具在执行时周期性检查这些字段，有则推送进度/响应取消。

## 替代方案

1. **修改 Tool trait** 返回 Stream — 入侵性大，影响所有现有工具
2. **外部进程管理**（HashMap<String, Child> 由执行器管理）— 需跨 crate 共享状态，复杂度高
3. **tokio-util CancellationToken** — 功能更强但引入新依赖

## 选择理由

- ToolContext 已经是 execute 的参数，加字段不影响现有工具
- `Option` 类型确保零开销（不使用时无性能损失）
- `Arc<AtomicBool>` 来自 std，无新增依赖
- 与现有的超时控制（timeout_secs）风格一致

## 影响

- 所有工具都可以选择性使用此机制，不影响已有工具
- execute engine 支持 `timeout_secs == 0` 表示不超时
- WebSocket handler 负责创建/转发进度通道及处理取消消息
