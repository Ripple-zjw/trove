const API_BASE = import.meta.env.VITE_API_BASE || 'http://127.0.0.1:8080';
const WS_BASE = API_BASE.replace(/^http/, 'ws');

export interface WsResult {
  type: 'result' | 'error' | 'pong';
  id?: string;
  data?: Record<string, unknown>;
  error?: string;
  code?: number;
}

type MessageCallback = (msg: WsResult) => void;
type StatusCallback = (status: 'connecting' | 'connected' | 'disconnected') => void;

export class ToolWebSocket {
  private ws: WebSocket | null = null;
  private onMessage: MessageCallback;
  private onStatus: StatusCallback;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private shouldReconnect = true;

  constructor(onMessage: MessageCallback, onStatus: StatusCallback) {
    this.onMessage = onMessage;
    this.onStatus = onStatus;
  }

  connect() {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    this.onStatus('connecting');
    this.ws = new WebSocket(`${WS_BASE}/api/ws`);

    this.ws.onopen = () => {
      this.onStatus('connected');
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data) as WsResult;
        this.onMessage(msg);
      } catch (e) {
        console.error('WS 消息解析失败:', e);
      }
    };

    this.ws.onclose = () => {
      this.onStatus('disconnected');
      if (this.shouldReconnect) {
        this.reconnectTimer = setTimeout(() => this.connect(), 3000);
      }
    };

    this.ws.onerror = () => {
      this.ws?.close();
    };
  }

  disconnect() {
    this.shouldReconnect = false;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.ws?.close();
    this.ws = null;
  }

  send(id: string, input: Record<string, unknown>) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'execute', id, input }));
    }
  }
}
