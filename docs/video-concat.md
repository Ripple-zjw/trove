# 视频拼接工具 (`video-concat`)

将多个视频文件拼接为一个视频文件。基于系统的 ffmpeg 实现。

## 依赖

需要系统已安装 ffmpeg：

```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt install ffmpeg

# Windows (choco)
choco install ffmpeg
```

可通过 `FFMPEG_PATH` 环境变量指定自定义 ffmpeg 路径。

## 使用方法

### CLI

```bash
# 基本用法（默认输出到 ~/Downloads/）
cargo run -- exec video-concat --input '{"files":["/path/to/a.mp4","/path/to/b.mp4"]}'

# 指定输出路径和画质
cargo run -- exec video-concat --input '{"files":["a.mp4","b.mov"],"output":"/tmp/merged.mp4","quality":"high"}'
```

### REST API

```bash
# 检测 ffmpeg
curl http://127.0.0.1:8080/api/tools/video-concat/deps

# 执行拼接
curl -X POST http://127.0.0.1:8080/api/tools/video-concat/execute \
  -H 'Content-Type: application/json' \
  -d '{"input":{"files":["/tmp/a.mp4","/tmp/b.mp4"],"quality":"medium"}}'
```

### WebSocket（支持进度推送和取消）

```json
// 发送执行
{"type":"execute","id":"video-concat","input":{"files":["a.mp4","b.mp4"]}}

// 收到进度
{"type":"progress","id":"video-concat","percent":0.45,"time":"00:00:12.30","frame":368,"speed":"1.5x"}

// 取消执行
{"type":"cancel","id":"video-concat"}
```

## 策略

| 条件 | 策略 | 说明 |
|------|------|------|
| 同容器格式 | concat demuxer | 流拷贝，无需重编码，极快 |
| 不同容器格式 | concat filter | H.264 重编码，支持跨格式拼接 |

## 输入参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `files` | `string[]` | 是 | 视频文件的绝对路径列表 |
| `output` | `string` | 否 | 输出路径，默认 `~/Downloads/video_concat_<时间戳>.mp4` |
| `quality` | `enum` | 否 | `low`(CRF28)、`medium`(CRF23)、`high`(CRF18)，默认 `medium` |
