import React, { useState, useEffect, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import { fetchTool, executeTool, ToolMetadata } from '../api/rest';

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

  // ═══════════════════════════════════════
  //  表单渲染（带 hover tooltip）
  // ═══════════════════════════════════════
  function renderForm() {
    if (!tool) return null;
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
          <button className="btn btn-primary" onClick={handleExecute} disabled={executing}>
            {executing ? '⏳ 执行中...' : '▶ 执行'}
          </button>
        </div>
      </div>

      {execError && <div className="error-box">{execError}</div>}
      {renderResult()}
    </div>
  );
}
