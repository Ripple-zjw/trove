import React, { useState, useEffect, useCallback, useRef } from 'react';
import { useParams, Link } from 'react-router-dom';
import { fetchTool, executeTool, ToolMetadata } from '../api/rest';
import { ToolWebSocket, WsResult } from '../api/ws';

const API_BASE = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:8080';

// ── 工具 ID 常量 ──
const TOOL_IDS = {
  JSON_FORMAT: 'json-format',
  JSON_VALIDATE: 'json-validate',
  STRING_CASE: 'string-case',
  BASE64_ENCODE: 'base64-encode',
  BASE64_DECODE: 'base64-decode',
  TS_TO_DATE: 'ts-to-date',
  DATE_TO_TS: 'date-to-ts',
  URL_ENCODE: 'url-encode',
  URL_DECODE: 'url-decode',
  UUID_GEN: 'uuid-gen',
  TEXT_STATS: 'text-stats',
  HASH: 'hash',
  VIDEO_CONCAT: 'video-concat',
} as const;

// ── 枚举选项的描述 ──
const ENUM_DESCRIPTIONS: Record<string, Record<string, string>> = {
  'to_case': {
    'lowercase': '全部转为小写字母，如 Hello → hello',
    'uppercase': '全部转为大写字母，如 Hello → HELLO',
    'capitalize': '仅首字母大写，其余小写，如 hello → Hello',
    'camelCase': '驼峰格式，单词首字母大写（首词小写），如 hello_world → helloWorld',
    'snake_case': '蛇形格式，单词用下划线连接，如 helloWorld → hello_world',
    'PascalCase': '帕斯卡格式，每个单词首字母大写，如 hello_world → HelloWorld',
    'kebab-case': '短横线格式，单词用连接号连接，如 helloWorld → hello-world',
    'CONSTANT_CASE': '常量格式，全大写+下划线，如 helloWorld → HELLO_WORLD',
  },
  'quality': {
    'low': '低画质（CRF 28），文件较小，编码速度快',
    'medium': '中等画质（CRF 23），较均衡（推荐）',
    'high': '高画质（CRF 18），文件较大，画质最好',
  },
};

export default function ToolExecute(): React.ReactElement | null {
  const { id } = useParams<{ id: string }>();
  const [tool, setTool] = useState<ToolMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [formValues, setFormValues] = useState<Record<string, unknown>>({});
  const [resultData, setResultData] = useState<Record<string, unknown> | null>(null);
  const [executing, setExecuting] = useState(false);
  const [execError, setExecError] = useState<string | null>(null);

  // ── video-concat 特有状态 ──
  const [ffmpegInfo, setFfmpegInfo] = useState<Record<string, unknown> | null>(null);
  const [ffmpegInfoError, setFfmpegInfoError] = useState(false);

  interface FileEntry {
    id: number;
    name: string;
    path: string;
    size: number;
    uploadedAt: number;
    uploading?: boolean;
  }
  const [fileEntries, setFileEntries] = useState<FileEntry[]>([]);
  const fileIdCounter = useRef(0);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const [dragOverIdx, setDragOverIdx] = useState<number | null>(null);
  const [dragOverContainer, setDragOverContainer] = useState(false);
  // 去重弹窗状态
  const [dupDialog, setDupDialog] = useState<{ files: File[]; names: string[] } | null>(null);
  // 下载状态
  const [downloading, setDownloading] = useState(false);
  const [sortBy, setSortBy] = useState<'name' | 'size' | 'time' | null>(null);
  const [sortAsc, setSortAsc] = useState(true);

  const [wsProgress, setWsProgress] = useState<number>(0);
  const [wsTime, setWsTime] = useState('');
  const [wsSpeed, setWsSpeed] = useState('');
  const wsRef = useRef<ToolWebSocket | null>(null);
  const cancelRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    if (!id) return;
    setLoading(true);
    setError(null);
    fetchTool(id)
      .then(t => {
        setTool(t);
        setFormValues(extractDefaults(t.input_schema));
      })
      .catch(e => setError(e.message))
      .finally(() => setLoading(false));

    // 视频拼接工具：检测 ffmpeg
    if (id === TOOL_IDS.VIDEO_CONCAT) {
      fetch(`${API_BASE}/api/tools/video-concat/deps`)
        .then(r => r.json())
        .then(info => {
          setFfmpegInfo(info as Record<string, unknown>);
          setFfmpegInfoError(false);
        })
        .catch(() => {
          setFfmpegInfoError(true);
        });
    }
  }, [id]);

  function extractDefaults(schema: Record<string, unknown>): Record<string, unknown> {
    const defaults: Record<string, unknown> = {};
    const properties = (schema.properties as Record<string, unknown>) || {};
    for (const [key, prop] of Object.entries(properties)) {
      const p = prop as Record<string, unknown>;
      if (p.default !== undefined) defaults[key] = p.default;
    }
    return defaults;
  }

  const handleExecute = useCallback(async () => {
    if (!id) return;
    setExecuting(true);
    setExecError(null);
    setResultData(null);
    try {
      const res = await executeTool(id, formValues);
      setResultData(res.result as Record<string, unknown>);
    } catch (e: unknown) {
      setExecError(e instanceof Error ? e.message : '执行失败');
    } finally {
      setExecuting(false);
    }
  }, [id, formValues]);

  function handleCopy(text: string) {
    navigator.clipboard.writeText(text).then(() => {
      const btn = document.activeElement as HTMLElement;
      if (btn) {
        const orig = btn.textContent;
        btn.textContent = '✓ 已复制';
        setTimeout(() => { btn.textContent = orig; }, 1500);
      }
    });
  }

  // ═══════════════════════════════════════
  //  结果渲染器：按工具类型定制展示
  // ═══════════════════════════════════════
  function renderResult() {
    if (!resultData || !id) return null;
    const d = resultData;

    switch (id) {
      // ──── JSON 格式化：展示格式化后的代码 ────
      case TOOL_IDS.JSON_FORMAT: {
        const text = d.result as string || '';
        return (
          <div className="result-section">
            <div className="result-header">
              <h3>📋 格式化结果</h3>
              <button className="copy-btn" onClick={() => handleCopy(text)}>复制</button>
            </div>
            <div className="result-body">
              <pre className="result-json">{text}</pre>
            </div>
          </div>
        );
      }

      // ──── JSON 校验 ────
      case TOOL_IDS.JSON_VALIDATE: {
        const valid = d.valid as boolean;
        if (valid) {
          return (
            <div className="result-section result-success">
              <div className="result-header">
                <h3>✅ JSON 校验通过</h3>
              </div>
              <div className="result-body">
                <table className="result-table">
                  <tbody>
                    <tr><td>类型</td><td>{(d.type as string) || '-'}</td></tr>
                    <tr><td>嵌套深度</td><td>{String(d.depth || 0)}</td></tr>
                    <tr><td>大小</td><td>{String(d.size || 0)} bytes</td></tr>
                  </tbody>
                </table>
              </div>
            </div>
          );
        }
        return (
          <div className="result-section result-error">
            <div className="result-header">
              <h3>❌ JSON 校验失败</h3>
            </div>
            <div className="result-body">
              <div className="error-badge">第 {String(d.line || '?')} 行，第 {String(d.column || '?')} 列</div>
              <pre className="error-detail">{(d.error as string) || '未知错误'}</pre>
            </div>
          </div>
        );
      }

      // ──── 字符串格式转换 ────
      case TOOL_IDS.STRING_CASE: {
        const original = d.original as string || '';
        const converted = d.result as string || '';
        const lbl = ENUM_DESCRIPTIONS.to_case?.[d.to_case as string] || '';
        return (
          <div className="result-section">
            <div className="result-header">
              <h3>🔤 转换结果</h3>
              <button className="copy-btn" onClick={() => handleCopy(converted)}>复制</button>
            </div>
            <div className="result-body">
              <div className="case-badge">{lbl}</div>
              <div className="case-compare">
                <div className="case-block">
                  <span className="case-label">原文本</span>
                  <code className="case-value case-original">{original}</code>
                </div>
                <div className="case-arrow">→</div>
                <div className="case-block">
                  <span className="case-label">转换后</span>
                  <code className="case-value case-converted">{converted}</code>
                </div>
              </div>
            </div>
          </div>
        );
      }

      // ──── Base64 ────
      case TOOL_IDS.BASE64_ENCODE:
      case TOOL_IDS.BASE64_DECODE: {
        const text = d.result as string || '';
        const isEncode = id === TOOL_IDS.BASE64_ENCODE;
        return (
          <div className="result-section">
            <div className="result-header">
              <h3>{isEncode ? '🔐 编码结果' : '🔓 解码结果'}</h3>
              <button className="copy-btn" onClick={() => handleCopy(text)}>复制</button>
            </div>
            <div className="result-body">
              <pre className="result-json result-mono">{text}</pre>
            </div>
          </div>
        );
      }

      // ──── 时间戳 → 日期 ────
      case TOOL_IDS.TS_TO_DATE: {
        return (
          <div className="result-section">
            <div className="result-header"><h3>📅 转换结果</h3></div>
            <div className="result-body">
              <div className="ts-card">
                <div className="ts-main">{d.result as string}</div>
                <div className="ts-meta">
                  <span>ISO 8601：{(d.iso_8601 as string) || '-'}</span>
                  <span>秒级时间戳：{String(d.timestamp_secs || 0)}</span>
                  <span>毫秒时间戳：{String(d.timestamp_ms || 0)}</span>
                </div>
              </div>
            </div>
          </div>
        );
      }

      // ──── 日期 → 时间戳 ────
      case TOOL_IDS.DATE_TO_TS: {
        return (
          <div className="result-section">
            <div className="result-header"><h3>📅 转换结果</h3></div>
            <div className="result-body">
              <div className="ts-card">
                <div className="ts-row">
                  <span className="ts-label">秒级时间戳</span>
                  <span className="ts-num">{String(d.timestamp_secs || 0)}</span>
                </div>
                <div className="ts-row">
                  <span className="ts-label">毫秒时间戳</span>
                  <span className="ts-num">{String(d.timestamp_ms || 0)}</span>
                </div>
              </div>
            </div>
          </div>
        );
      }

      // ──── URL 编解码 ────
      case TOOL_IDS.URL_ENCODE:
      case TOOL_IDS.URL_DECODE: {
        const text = d.result as string || '';
        const isEncode = id === TOOL_IDS.URL_ENCODE;
        return (
          <div className="result-section">
            <div className="result-header">
              <h3>🔗 {isEncode ? '编码' : '解码'}结果</h3>
              <button className="copy-btn" onClick={() => handleCopy(text)}>复制</button>
            </div>
            <div className="result-body">
              <pre className="result-mono">{text}</pre>
            </div>
          </div>
        );
      }

      // ──── UUID 生成 ────
      case TOOL_IDS.UUID_GEN: {
        const uuids = (d.uuids as string[]) || [];
        const count = d.count as number || 0;
        return (
          <div className="result-section">
            <div className="result-header"><h3>🆔 已生成 {count} 个 UUID</h3></div>
            <div className="result-body">
              <div className="uuid-list">
                {uuids.map((uuid: string, i: number) => (
                  <div key={i} className="uuid-item">
                    <span className="uuid-index">{i + 1}.</span>
                    <code className="uuid-value">{uuid}</code>
                    <button className="copy-btn-sm" onClick={() => handleCopy(uuid)}>复制</button>
                  </div>
                ))}
              </div>
            </div>
          </div>
        );
      }

      // ──── 文本统计 ────
      case TOOL_IDS.TEXT_STATS: {
        const freq = d.word_frequency as Array<Record<string, unknown>>;
        return (
          <div className="result-section">
            <div className="result-header"><h3>📊 文本统计</h3></div>
            <div className="result-body">
              <table className="result-table">
                <tbody>
                  <tr><td>字符数</td><td>{String(d.char_count || 0)}</td></tr>
                  <tr><td>非空格字符</td><td>{String(d.char_count_no_space || 0)}</td></tr>
                  <tr><td>单词数</td><td>{String(d.word_count || 0)}</td></tr>
                  <tr><td>行数</td><td>{String(d.line_count || 0)}</td></tr>
                  <tr><td>字节数</td><td>{String(d.byte_count || 0)}</td></tr>
                  <tr><td>CJK 字符</td><td>{String(d.cjk_char_count || 0)}</td></tr>
                  <tr><td>字母数</td><td>{String(d.letter_count || 0)}</td></tr>
                  <tr><td>数字数</td><td>{String(d.digit_count || 0)}</td></tr>
                </tbody>
              </table>
              {freq && freq.length > 0 && (
                <>
                  <h4 style={{ marginTop: 16, marginBottom: 8, fontSize: 14 }}>词频统计</h4>
                  <div className="freq-list">
                    {freq.slice(0, 20).map((item, i) => {
                      const word = item.word as string;
                      const count = item.count as number;
                      const barW = Math.min(count * 20, 300);
                      return (
                        <div key={i} className="freq-row">
                          <span className="freq-word">{word}</span>
                          <div className="freq-bar" style={{ width: barW }} />
                          <span className="freq-count">{count}</span>
                        </div>
                      );
                    })}
                  </div>
                </>
              )}
            </div>
          </div>
        );
      }

      // ──── Hash ────
      case TOOL_IDS.HASH: {
        const hash = d.result as string || '';
        return (
          <div className="result-section">
            <div className="result-header">
              <h3>🔑 {(d.algorithm as string) || ''} 哈希值</h3>
              <button className="copy-btn" onClick={() => handleCopy(hash)}>复制</button>
            </div>
            <div className="result-body">
              <pre className="result-mono result-hash">{hash}</pre>
              <div className="hash-meta">输入长度：{String(d.input_length || 0)} 字符</div>
            </div>
          </div>
        );
      }

      // ──── 视频拼接 ────
      case TOOL_IDS.VIDEO_CONCAT: {
        return renderVideoConcatResult();
      }

      // ──── 通用兜底 ────
      default: {
        const text = JSON.stringify(resultData, null, 2);
        return (
          <div className="result-section">
            <div className="result-header">
              <h3>执行结果</h3>
              <button className="copy-btn" onClick={() => handleCopy(text)}>复制</button>
            </div>
            <div className="result-body">
              <pre className="result-json">{text}</pre>
            </div>
          </div>
        );
      }
    }
  }

  // ── 文件操作 ────────────────────────────────

  /** 上传单个文件到后端 */
  async function uploadFile(file: File): Promise<FileEntry | null> {
    const formData = new FormData();
    formData.append('file', file, file.name);
    try {
      const res = await fetch(`${API_BASE}/api/upload`, {
        method: 'POST',
        body: formData,
      });
      if (!res.ok) {
        const errText = await res.text();
        throw new Error(errText || `HTTP ${res.status}`);
      }
      const data = await res.json() as { path: string; name: string; size: number };
      return {
        id: ++fileIdCounter.current,
        name: data.name,
        path: data.path,
        size: data.size,
        uploadedAt: Date.now(),
      };
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error(`上传 "${file.name}" 失败:`, e);
      setExecError(`上传文件 "${file.name}" 失败: ${msg}`);
      return null;
    }
  }

  /** 上传多个文件 */
  async function uploadFiles(files: FileList | File[], skipDuplicates: boolean = false) {
    const existingNames = new Set(fileEntries.map(e => e.name));
    const fileArr = Array.from(files);
    const dupNames: string[] = [];
    const unique: File[] = [];

    for (const f of fileArr) {
      if (existingNames.has(f.name)) {
        dupNames.push(f.name);
      } else {
        unique.push(f);
      }
    }

    // 有重复文件，且未决定跳过
    if (dupNames.length > 0 && !skipDuplicates) {
      setDupDialog({ files: fileArr, names: dupNames });
      return;
    }

    // 跳过重复，只上传不重复的
    const toUpload = skipDuplicates ? unique : fileArr;

    const newEntries: FileEntry[] = [];
    for (const f of toUpload) {
      const entry = await uploadFile(f);
      if (entry) newEntries.push(entry);
    }
    if (newEntries.length > 0) {
      setFileEntries(prev => [...prev, ...newEntries]);
    }
    setDupDialog(null);
  }

  /** 打开文件选择器 */
  function handlePickFiles() {
    const input = document.createElement('input');
    input.type = 'file';
    input.multiple = true;
    input.accept = 'video/*,.mp4,.mov,.mkv,.webm,.avi,.flv,.wmv';
    input.onchange = async () => {
      if (input.files && input.files.length > 0) {
        await uploadFiles(input.files);
      }
    };
    input.click();
  }

  /** 拖拽事件：阻止浏览器默认行为 */
  function handleDragOver(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = 'copy';
    setDragOverContainer(true);
  }

  /** 拖拽离开容器 */
  function handleDragLeave(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setDragOverContainer(false);
  }

  /** 拖拽放入文件 */
  function handleDrop(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    setDragOverContainer(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      uploadFiles(e.dataTransfer.files);
    }
  }

  /** 排序文件 */
  function sortFiles(by: 'name' | 'size' | 'time') {
    const sorted = [...fileEntries];
    if (by === sortBy) {
      // 切换升降序
      sorted.reverse();
      setSortAsc(!sortAsc);
    } else {
      sorted.sort((a, b) => {
        if (by === 'name') return a.name.localeCompare(b.name);
        if (by === 'size') return a.size - b.size;
        return a.uploadedAt - b.uploadedAt;
      });
      setSortBy(by);
      setSortAsc(true);
    }
    setFileEntries(sorted);
  }

  /** 下载文件到用户本地 */
  async function downloadOutput(outputPath: string, fileName: string) {
    setDownloading(true);
    try {
      const res = await fetch(`${API_BASE}/api/download?path=${encodeURIComponent(outputPath)}`);
      if (!res.ok) throw new Error('下载失败');
      const blob = await res.blob();

      // 优先使用 showSaveFilePicker API
      if ('showSaveFilePicker' in window) {
        const handle = await (window as any).showSaveFilePicker({
          suggestedName: fileName,
          types: [{
            description: '视频文件',
            accept: { 'video/mp4': ['.mp4'], 'video/quicktime': ['.mov'], 'video/x-matroska': ['.mkv'], 'video/webm': ['.webm'] },
          }],
        });
        const writable = await handle.createWritable();
        await writable.write(blob);
        await writable.close();
      } else {
        // 回退：触发浏览器默认下载
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = fileName;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        setTimeout(() => URL.revokeObjectURL(url), 1000);
      }
    } catch (e) {
      if ((e as Error).name !== 'AbortError') {
        setExecError(`下载失败: ${(e as Error).message}`);
      }
    } finally {
      setDownloading(false);
    }
  }

  /** 格式化文件大小 */
  function formatSize(bytes: number): string {
    if (bytes === 0) return '—';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  // ═══════════════════════════════════════
  //  video-concat 表单
  // ═══════════════════════════════════════
  function renderVideoConcatForm() {
    const quality = (formValues['quality'] as string) || 'medium';
    const outputFormat = (formValues['output_format'] as string) || 'mp4';
    const resolution = (formValues['resolution'] as string) || 'original';

    const ffmpegStatus = () => {
      if (ffmpegInfoError) return <span className="ffmpeg-badge unavailable">⚠️ 无法检测 ffmpeg（服务器 API 不可用）</span>;
      if (ffmpegInfo === null) return <span className="ffmpeg-badge" style={{ opacity: 0.5 }}>⏳ 检测中...</span>;
      if (ffmpegInfo?.available) {
        return (
          <>
            <span className="ffmpeg-badge available">✅ ffmpeg 已就绪 — {ffmpegInfo.version as string || ''}</span>
            <p className="form-hint">路径：{ffmpegInfo.path as string}</p>
            <p className="form-hint">本工具依赖系统中的 ffmpeg。不同格式的视频拼接时自动转码为 H.264（推荐），同格式则使用流拷贝（无损）。</p>
          </>
        );
      }
      return (
        <>
          <span className="ffmpeg-badge unavailable">❌ 系统中未找到 ffmpeg</span>
          <p className="form-hint" style={{ marginTop: 6 }}>
            本工具需要 ffmpeg 来处理视频。请安装：<br />
            macOS：<code>brew install ffmpeg</code><br />
            Ubuntu：<code>sudo apt install ffmpeg</code><br />
            Windows：<code>choco install ffmpeg</code><br />
            或设置 <code>FFMPEG_PATH</code> 环境变量指定自定义路径。
          </p>
        </>
      );
    };

    return (
      <>
        {/* ffmpeg 依赖说明 */}
        <div className="form-group">
          <label className="form-label">🎥 依赖检查</label>
          {ffmpegStatus()}
        </div>

        {/* 文件选择区 */}
        <div className="form-group">
          <div className="file-list-header">
            <label className="form-label" style={{ marginBottom: 0 }}>📁 视频文件列表 <span className="required-mark">*</span></label>
            <div className="file-toolbar">
              <button className="btn-sm" onClick={handlePickFiles}>📂 选择文件</button>
              {fileEntries.length > 1 && (
                <>
                  <button className="btn-sm" onClick={() => sortFiles('name')}>
                    按名称{sortBy === 'name' ? (sortAsc ? '↑' : '↓') : ''}
                  </button>
                  <button className="btn-sm" onClick={() => sortFiles('size')}>
                    按大小{sortBy === 'size' ? (sortAsc ? '↑' : '↓') : ''}
                  </button>
                  <button className="btn-sm" onClick={() => sortFiles('time')}>
                    按时间{sortBy === 'time' ? (sortAsc ? '↑' : '↓') : ''}
                  </button>
                </>
              )}
            </div>
          </div>

          <div
            className={`file-list ${dragOverContainer ? 'file-list-dropzone' : ''}`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            {fileEntries.length === 0 && (
              <div className="file-list-empty" onClick={handlePickFiles}>
                📂 拖拽视频文件到此，或点击选择文件
              </div>
            )}
            {fileEntries.map((entry, i) => (
              <div
                key={entry.id}
                className={`file-list-item ${dragOverIdx === i ? 'drag-over' : ''} ${dragIdx === i ? 'dragging' : ''}`}
                draggable
                onDragStart={(e) => {
                  setDragIdx(i);
                  e.dataTransfer.effectAllowed = 'move';
                  e.stopPropagation();
                }}
                onDragOver={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  setDragOverIdx(i);
                }}
                onDragLeave={(e) => {
                  e.stopPropagation();
                  setDragOverIdx(null);
                }}
                onDrop={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  if (dragIdx === null || dragIdx === i) {
                    setDragIdx(null);
                    setDragOverIdx(null);
                    return;
                  }
                  const list = [...fileEntries];
                  const [removed] = list.splice(dragIdx, 1);
                  list.splice(i, 0, removed);
                  setFileEntries(list);
                  setDragIdx(null);
                  setDragOverIdx(null);
                }}
                onDragEnd={(e) => {
                  e.stopPropagation();
                  setDragIdx(null);
                  setDragOverIdx(null);
                }}
              >
                <span className="file-drag-handle">⠿</span>
                <span className="file-index">{i + 1}.</span>
                <span className="file-name">{entry.name}</span>
                <span className="file-size">{formatSize(entry.size)}</span>
                <button
                  className="btn-sm btn-danger"
                  onClick={() => setFileEntries(fileEntries.filter((_, idx) => idx !== i))}
                  title="移除"
                >✕</button>
              </div>
            ))}
          </div>
          {fileEntries.length > 0 && (
            <p className="form-hint">
              已选择 {fileEntries.length} 个文件。拖拽文件名可调整顺序。共 {formatSize(fileEntries.reduce((s, e) => s + e.size, 0))}。
              最多 200 个文件。
            </p>
          )}
        </div>

        {/* 输出设置 */}
        <div className="form-group form-row-group">
          <div className="form-row">
            <div className="form-row-item">
              <label className="form-label" htmlFor="f-format">输出格式</label>
              <select
                id="f-format"
                className="form-select"
                value={outputFormat}
                onChange={e => setFormValues(v => ({ ...v, output_format: e.target.value }))}
              >
                <option value="mp4">MP4 (推荐，最通用)</option>
                <option value="mov">MOV (QuickTime)</option>
                <option value="mkv">MKV (Matroska)</option>
                <option value="webm">WebM (Web)</option>
                <option value="avi">AVI (兼容老设备)</option>
              </select>
            </div>
            <div className="form-row-item">
              <label className="form-label" htmlFor="f-resolution">分辨率</label>
              <select
                id="f-resolution"
                className="form-select"
                value={resolution}
                onChange={e => setFormValues(v => ({ ...v, resolution: e.target.value }))}
              >
                <option value="original">原始分辨率（推荐）</option>
                <option value="1080p">1080p（全高清）</option>
                <option value="720p">720p（高清）</option>
                <option value="480p">480p（标清）</option>
                <option value="360p">360p（流畅）</option>
              </select>
            </div>
            <div className="form-row-item">
              <label className="form-label" htmlFor="f-quality">画质</label>
              <select
                id="f-quality"
                className="form-select"
                value={quality}
                onChange={e => setFormValues(v => ({ ...v, quality: e.target.value }))}
              >
                <option value="low">低 (CRF 28，文件小)</option>
                <option value="medium">中 (CRF 23，均衡，推荐)</option>
                <option value="high">高 (CRF 18，画质好)</option>
              </select>
            </div>
          </div>
        </div>

        {/* 输出提示 */}
        <p className="form-hint" style={{ marginTop: 8 }}>
          拼接完成后，可通过「保存到本地」按钮将结果下载到你的电脑。
        </p>
      </>
    );
  }

  // ═══════════════════════════════════════
  //  video-concat 结果渲染（带进度条）
  // ═══════════════════════════════════════
  function renderVideoConcatResult() {
    if (executing) {
      const pct = Math.min(Math.round(wsProgress * 100), 99);
      return (
        <div className="result-section">
          <div className="result-header"><h3>🎬 正在拼接视频...</h3></div>
          <div className="result-body">
            <div className="progress-bar-container">
              <div className="progress-bar-fill" style={{ width: `${pct}%` }} />
              <div className="progress-bar-label">{pct > 0 ? `${pct}%` : '准备中...'}</div>
            </div>
            <div className="progress-details">
              {wsTime && <span>当前处理位置：{wsTime}</span>}
              {wsSpeed && <span>速度：{wsSpeed}</span>}
            </div>
            <button className="btn btn-danger" onClick={() => cancelRef.current?.()} style={{ marginTop: 12 }}>
              ⏹ 取消拼接
            </button>
          </div>
        </div>
      );
    }

    if (!resultData) return null;
    const d = resultData;
    const success = d.success as boolean;
    const cancelled = d.cancelled as boolean;

    if (cancelled) {
      return (
        <div className="result-section">
          <div className="result-header"><h3>⏹ 已取消</h3></div>
          <div className="result-body"><p>{(d.message as string) || '用户取消了拼接操作'}</p></div>
        </div>
      );
    }

    if (success) {
      const size = d.output_size_bytes as number;
      const sizeStr = size > 1024 * 1024
        ? `${(size / (1024 * 1024)).toFixed(2)} MB`
        : `${size} bytes`;
      const strategy = d.strategy === 'stream-copy'
        ? '流拷贝（无损，同格式直接拼接）'
        : '重编码（H.264，跨格式兼容拼接）';
      const outPath = (d.output_path as string) || '';
      const outName = outPath.split('/').pop() || 'output.mp4';

      return (
        <div className="result-section result-success">
          <div className="result-header"><h3>✅ 视频拼接完成</h3></div>
          <div className="result-body">
            <table className="result-table">
              <tbody>
                <tr><td>输入文件数</td><td>{String(d.input_count || 0)}</td></tr>
                <tr><td>拼接策略</td><td>{strategy}</td></tr>
                <tr><td>输出大小</td><td>{sizeStr}</td></tr>
                <tr><td>时长</td><td>{d.output_duration_secs ? `${Number(d.output_duration_secs).toFixed(1)} 秒` : '-'}</td></tr>
                <tr><td>ffmpeg</td><td style={{ fontSize: 12 }}>{`${d.ffmpeg_version as string || ''}`}</td></tr>
              </tbody>
            </table>
            <div style={{ marginTop: 16 }}>
              <button
                className="btn btn-primary"
                onClick={() => downloadOutput(outPath, outName)}
                disabled={downloading}
              >
                {downloading ? '⏳ 下载中...' : '💾 保存到本地'}
              </button>
            </div>
          </div>
        </div>
      );
    }

    return null;
  }

  // ═══════════════════════════════════════
  //  video-concat 执行（使用 WebSocket）
  // ═══════════════════════════════════════
  function handleVideoConcatExecute() {
    const validFiles = fileEntries.filter(e => e.path);
    if (validFiles.length === 0) {
      setExecError('请至少添加一个视频文件');
      return;
    }

    setExecuting(true);
    setExecError(null);
    setResultData(null);
    setWsProgress(0);
    setWsTime('');
    setWsSpeed('');

    const ws = new ToolWebSocket(
      (msg: WsResult) => {
        if (msg.type === 'progress' && msg.percent !== undefined) {
          setWsProgress(msg.percent);
          setWsTime(msg.time || '');
          setWsSpeed(msg.speed || '');
        } else if (msg.type === 'result' && msg.data) {
          setResultData(msg.data);
          setExecuting(false);
          ws.disconnect();
        } else if (msg.type === 'error') {
          setExecError(msg.error || '执行失败');
          setExecuting(false);
          ws.disconnect();
        }
      },
      () => {}
    );

    ws.connect();

    const timeout = setTimeout(() => {
      const input: Record<string, unknown> = {
        files: validFiles.map(e => e.path),
        output_format: formValues['output_format'] || 'mp4',
        resolution: formValues['resolution'] || 'original',
        quality: formValues['quality'] || 'medium',
      };
      if (formValues['output']) {
        input['output'] = formValues['output'];
      }
      ws.send(TOOL_IDS.VIDEO_CONCAT, input);
      cancelRef.current = () => {
        ws.cancel(TOOL_IDS.VIDEO_CONCAT);
      };
    }, 300);

    wsRef.current = ws;
    return () => {
      clearTimeout(timeout);
      ws.disconnect();
    };
  }

  // ═══════════════════════════════════════
  //  表单渲染（带 hover tooltip）
  // ═══════════════════════════════════════
  function renderForm() {
    if (!tool) return null;

    // video-concat 使用定制表单
    if (id === TOOL_IDS.VIDEO_CONCAT) {
      return renderVideoConcatForm();
    }

    const schema = tool.input_schema;
    const properties = (schema.properties as Record<string, unknown>) || {};
    const required = (schema.required as string[]) || [];

    return Object.entries(properties).map(([key, prop]) => {
      const p = prop as Record<string, unknown>;
      const title = (p.title as string) || key;
      const desc = (p.description as string) || '';
      const propType = p.type as string;
      const isRequired = required.includes(key);
      const isEnum = Array.isArray(p.enum);
      const value = formValues[key] ?? '';
      const enumDescs = tool.id ? ENUM_DESCRIPTIONS[key] : undefined;

      const label = (
        <label className="form-label" htmlFor={`f-${key}`} title={desc}>
          {title}
          {isRequired && <span className="required-mark">*</span>}
          {desc && <span className="tooltip-icon" title={desc}>ⓘ</span>}
        </label>
      );

      let input: React.ReactNode;

      if (isEnum) {
        input = (
          <select
            id={`f-${key}`}
            className="form-select"
            value={String(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value }))}
          >
            {(p.enum as string[]).map(opt => (
              <option
                key={opt}
                value={opt}
                title={enumDescs?.[opt] || ''}
              >
                {opt}
                {enumDescs?.[opt] ? ` — ${enumDescs[opt].split('，')[0]}` : ''}
              </option>
            ))}
          </select>
        );
        // 显示当前选中项的详细说明
        const currDesc = enumDescs?.[String(value)];
        return (
          <div key={key} className="form-group">
            {label}
            {input}
            {currDesc && <p className="enum-hint">{currDesc}</p>}
          </div>
        );
      }

      if (propType === 'boolean') {
        return (
          <div key={key} className="form-group">
            <label className="form-checkbox" title={desc}>
              <input
                type="checkbox"
                checked={Boolean(value)}
                onChange={e => setFormValues(v => ({ ...v, [key]: e.target.checked }))}
              />
              {title}
            </label>
          </div>
        );
      }

      if (propType === 'integer' || propType === 'number') {
        input = (
          <input
            id={`f-${key}`}
            type="number"
            className="form-input"
            value={value === '' ? '' : Number(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value === '' ? '' : Number(e.target.value) }))}
            min={p.minimum as number}
            max={p.maximum as number}
            title={desc}
          />
        );
      } else if (propType === 'string' && key === 'input') {
        input = (
          <textarea
            id={`f-${key}`}
            className="form-textarea"
            value={String(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value }))}
            placeholder={desc}
            rows={6}
            title={desc}
          />
        );
      } else {
        input = (
          <input
            id={`f-${key}`}
            type="text"
            className="form-input"
            value={String(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value }))}
            placeholder={desc}
            title={desc}
          />
        );
      }

      return (
        <div key={key} className="form-group">
          {label}
          {input}
        </div>
      );
    });
  }

  // ═══════════════════════════════════════
  //  页面主体
  // ═══════════════════════════════════════
  if (loading) return <div className="empty-state"><p>加载中...</p></div>;

  if (error) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">⚠️</div>
        <p className="empty-state-text">{error}</p>
        <Link to="/" className="btn btn-primary" style={{ marginTop: 16, display: 'inline-flex' }}>返回工具列表</Link>
      </div>
    );
  }

  if (!tool) return null;

  const category = (tool.category as unknown as { label: string }).label || '';

  return (
    <div className="execute-page">
      <Link to="/" className="back-link">← 返回工具列表</Link>

      <div className="execute-header">
        <div className="tool-name">{tool.name}</div>
        <div className="tool-desc">{tool.description} · {category}</div>
      </div>

      <div className="form-section">
        {renderForm()}
        <div className="form-group" style={{ marginTop: 20 }}>
          <button
            className="btn btn-primary"
            onClick={id === TOOL_IDS.VIDEO_CONCAT ? handleVideoConcatExecute : handleExecute}
            disabled={executing}
          >
            {executing ? (id === TOOL_IDS.VIDEO_CONCAT ? '🎬 拼接中...' : '⏳ 执行中...') : '▶ 执行'}
          </button>
        </div>
      </div>

      {execError && <div className="error-box">{execError}</div>}
      {renderResult()}

      {/* 去重弹窗 */}
      {dupDialog && (
        <div className="modal-overlay" onClick={() => setDupDialog(null)}>
          <div className="modal-box" onClick={e => e.stopPropagation()}>
            <h3 className="modal-title">⚠️ 有重复文件</h3>
            <p className="modal-desc">
              以下 {dupDialog.names.length} 个文件已经存在于列表中：
            </p>
            <ul className="modal-dup-list">
              {dupDialog.names.map(n => <li key={n}>{n}</li>)}
            </ul>
            <div className="modal-actions">
              <button
                className="btn btn-primary"
                onClick={() => {
                  // 跳过重复，只添加新文件
                  uploadFiles(dupDialog.files, true);
                }}
              >跳过重复，只添加新文件</button>
              <button
                className="btn"
                onClick={() => {
                  // 仍要添加重复文件
                  setDupDialog(null);
                  const fileArr = Array.from(dupDialog.files);
                  (async () => {
                    const newEntries: FileEntry[] = [];
                    for (const f of fileArr) {
                      const entry = await uploadFile(f);
                      if (entry) newEntries.push(entry);
                    }
                    if (newEntries.length > 0) {
                      setFileEntries(prev => [...prev, ...newEntries]);
                    }
                  })();
                }}
              >全部添加（含重复）</button>
              <button
                className="btn"
                onClick={() => setDupDialog(null)}
              >取消</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
