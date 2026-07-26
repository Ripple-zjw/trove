import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { fetchTools, ToolMetadata } from '../api/rest';

interface CategoryGroup {
  label: string;
  tools: ToolMetadata[];
}

export default function ToolList() {
  const [groups, setGroups] = useState<CategoryGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const navigate = useNavigate();

  useEffect(() => {
    loadTools();
  }, []);

  async function loadTools() {
    try {
      setLoading(true);
      const data = await fetchTools();
      const categorized = groupByCategory(data.tools);
      setGroups(categorized);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : '加载失败');
    } finally {
      setLoading(false);
    }
  }

  function groupByCategory(tools: ToolMetadata[]): CategoryGroup[] {
    const map = new Map<string, ToolMetadata[]>();
    for (const tool of tools) {
      const catLabel = (tool.category as unknown as { label: string }).label || '其他';
      if (!map.has(catLabel)) map.set(catLabel, []);
      map.get(catLabel)!.push(tool);
    }
    return Array.from(map.entries()).map(([label, tools]) => ({ label, tools }));
  }

  const filteredGroups = groups
    .map(g => ({
      ...g,
      tools: g.tools.filter(t =>
        t.name.includes(search) ||
        t.id.includes(search) ||
        t.description.includes(search)
      ),
    }))
    .filter(g => g.tools.length > 0);

  if (loading) {
    return <div className="empty-state"><p>加载中...</p></div>;
  }

  if (error) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">⚠️</div>
        <p className="empty-state-text">无法连接到 Trove 服务</p>
        <p style={{ fontSize: 13, color: 'var(--text-secondary)', marginTop: 8 }}>{error}</p>
        <button className="btn btn-primary" style={{ marginTop: 16 }} onClick={loadTools}>
          重试
        </button>
      </div>
    );
  }

  return (
    <div>
      <h2 className="page-title">工具集</h2>
      <p className="page-desc">选择工具开始使用</p>

      <input
        type="text"
        className="form-input"
        placeholder="搜索工具..."
        value={search}
        onChange={e => setSearch(e.target.value)}
        style={{ marginBottom: 24 }}
      />

      {filteredGroups.map(group => (
        <div key={group.label} className="category-section">
          <h3 className="category-title">{group.label}</h3>
          <div className="tool-grid">
            {group.tools.map(tool => (
              <div
                key={tool.id}
                className="tool-card"
                onClick={() => navigate(`/tool/${tool.id}`)}
              >
                <div className="tool-card-name">{tool.name}</div>
                <div className="tool-card-desc">{tool.description}</div>
              </div>
            ))}
          </div>
        </div>
      ))}

      {filteredGroups.length === 0 && (
        <div className="empty-state">
          <p className="empty-state-text">未找到匹配的工具</p>
        </div>
      )}
    </div>
  );
}
