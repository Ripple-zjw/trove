const API_BASE = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:8080';

export interface ToolCategory {
  label: string;
  order: number;
}

export interface ToolMetadata {
  id: string;
  name: string;
  description: string;
  category: ToolCategory;
  input_schema: Record<string, unknown>;
  is_cpu_intensive: boolean;
}

export interface ToolListResponse {
  tools: ToolMetadata[];
  total: number;
}

export interface ExecuteResponse {
  result: Record<string, unknown>;
}

export interface ErrorResponse {
  error: string;
  code: number;
}

export async function fetchTools(): Promise<ToolListResponse> {
  const res = await fetch(`${API_BASE}/api/tools`);
  if (!res.ok) throw new Error(`获取工具列表失败: ${res.status}`);
  return res.json();
}

export async function fetchTool(id: string): Promise<ToolMetadata> {
  const res = await fetch(`${API_BASE}/api/tools/${encodeURIComponent(id)}`);
  if (!res.ok) {
    if (res.status === 404) throw new Error(`工具 "${id}" 未找到`);
    throw new Error(`获取工具信息失败: ${res.status}`);
  }
  return res.json();
}

export async function executeTool(
  id: string,
  input: Record<string, unknown>
): Promise<ExecuteResponse> {
  const res = await fetch(`${API_BASE}/api/tools/${encodeURIComponent(id)}/execute`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ input }),
  });
  if (!res.ok) {
    const errBody = await res.text();
    let errorMsg = `请求失败: ${res.status}`;
    try {
      const errJson = JSON.parse(errBody) as ErrorResponse;
      errorMsg = errJson.error;
    } catch {
      // ignore parse error
    }
    throw new Error(errorMsg);
  }
  return res.json();
}
