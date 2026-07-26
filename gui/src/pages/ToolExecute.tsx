import React, { useState, useEffect, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import { fetchTool, executeTool, ToolMetadata } from '../api/rest';

export default function ToolExecute(): React.ReactElement | null {
  const { id } = useParams<{ id: string }>();
  const [tool, setTool] = useState<ToolMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [formValues, setFormValues] = useState<Record<string, unknown>>({});
  const [result, setResult] = useState<string | null>(null);
  const [executing, setExecuting] = useState(false);
  const [execError, setExecError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!id) return;
    setLoading(true);
    setError(null);
    fetchTool(id)
      .then(t => {
        setTool(t);
        // Initialize form with default values from schema
        const defaults = extractDefaults(t.input_schema);
        setFormValues(defaults);
      })
      .catch(e => setError(e.message))
      .finally(() => setLoading(false));
  }, [id]);

  function extractDefaults(schema: Record<string, unknown>): Record<string, unknown> {
    const defaults: Record<string, unknown> = {};
    const properties = (schema.properties as Record<string, unknown>) || {};
    for (const [key, prop] of Object.entries(properties)) {
      const propObj = prop as Record<string, unknown>;
      if (propObj.default !== undefined) {
        defaults[key] = propObj.default;
      }
    }
    return defaults;
  }

  const handleExecute = useCallback(async () => {
    if (!id) return;
    setExecuting(true);
    setExecError(null);
    setResult(null);
    try {
      const res = await executeTool(id, formValues);
      setResult(JSON.stringify(res.result, null, 2));
    } catch (e: unknown) {
      setExecError(e instanceof Error ? e.message : '执行失败');
    } finally {
      setExecuting(false);
    }
  }, [id, formValues]);

  function handleCopy() {
    if (!result) return;
    navigator.clipboard.writeText(result).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  // Render form fields dynamically from JSON Schema
  function renderFormFields() {
    if (!tool) return null;
    const schema = tool.input_schema;
    const properties = (schema.properties as Record<string, unknown>) || {};
    const required = (schema.required as string[]) || [];

    return Object.entries(properties).map(([key, prop]) => {
      const propObj = prop as Record<string, unknown>;
      const title = (propObj.title as string) || key;
      const desc = (propObj.description as string) || '';
      const propType = propObj.type as string;
      const isRequired = required.includes(key);
      const isEnum = Array.isArray(propObj.enum);
      const value = formValues[key] ?? '';

      const label = (
        <label className="form-label" htmlFor={`field-${key}`}>
          {title}
          {isRequired && <span style={{ color: 'var(--danger)', marginLeft: 2 }}>*</span>}
        </label>
      );

      const description = desc ? <p className="form-description">{desc}</p> : null;

      let input: React.ReactNode;

      if (isEnum) {
        input = (
          <select
            id={`field-${key}`}
            className="form-select"
            value={String(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value }))}
          >
            {(propObj.enum as string[]).map(opt => (
              <option key={opt} value={opt}>{opt}</option>
            ))}
          </select>
        );
      } else if (propType === 'boolean') {
        input = (
          <label className="form-checkbox">
            <input
              type="checkbox"
              checked={Boolean(value)}
              onChange={e => setFormValues(v => ({ ...v, [key]: e.target.checked }))}
            />
            {title}
          </label>
        );
      } else if (propType === 'integer' || propType === 'number') {
        input = (
          <input
            id={`field-${key}`}
            type="number"
            className="form-input"
            value={value === '' ? '' : Number(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value === '' ? '' : Number(e.target.value) }))}
            min={propObj.minimum as number}
            max={propObj.maximum as number}
          />
        );
      } else if (propType === 'string' && key === 'input') {
        input = (
          <textarea
            id={`field-${key}`}
            className="form-textarea"
            value={String(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value }))}
            placeholder={desc}
            rows={6}
          />
        );
      } else {
        input = (
          <input
            id={`field-${key}`}
            type="text"
            className="form-input"
            value={String(value)}
            onChange={e => setFormValues(v => ({ ...v, [key]: e.target.value }))}
            placeholder={desc}
          />
        );
      }

      if (propType === 'boolean') {
        return <div key={key} className="form-group">{input}</div>;
      }

      return (
        <div key={key} className="form-group">
          {label}
          {description}
          {input}
        </div>
      );
    });
  }

  if (loading) {
    return <div className="empty-state"><p>加载中...</p></div>;
  }

  if (error) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">⚠️</div>
        <p className="empty-state-text">{error}</p>
        <Link to="/" className="btn btn-primary" style={{ marginTop: 16, display: 'inline-flex' }}>
          返回工具列表
        </Link>
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
        {renderFormFields()}
        <div className="form-group" style={{ marginTop: 20 }}>
          <button
            className="btn btn-primary"
            onClick={handleExecute}
            disabled={executing}
          >
            {executing ? '执行中...' : '执行'}
          </button>
        </div>
      </div>

      {execError && (
        <div className="error-box">{execError}</div>
      )}

      {result && (
        <div className="result-section">
          <div className="result-header">
            <h3>执行结果</h3>
            <button className="copy-btn" onClick={handleCopy}>
              {copied ? '已复制' : '复制'}
            </button>
          </div>
          <div className="result-body">
            <pre className="result-json">{result}</pre>
          </div>
        </div>
      )}
    </div>
  );
}
