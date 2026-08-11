import { WebSocketMessage } from '../types/websocket';

type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

class WebSocketService {
  private ws: WebSocket | null = null;
  private url: string;
  private token: string | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectInterval = 1000; // Initial reconnect interval in ms

  public onMessage: ((message: WebSocketMessage) => void) | null = null;
  public onStatusChange: ((status: ConnectionStatus) => void) | null = null;

  constructor(url: string) {
    this.url = url;
  }

  public connect(token?: string): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      return;
    }

    if (token) {
      this.token = token;
    }

    const fullUrl = this.token ? `${this.url}?token=${this.token}` : this.url;
    console.log('Attempting WebSocket connection to:', fullUrl);

    this.ws = new WebSocket(fullUrl);
    this.setStatus('connecting');

    this.ws.onopen = () => {
      console.log('WebSocket connection opened successfully');
      this.setStatus('connected');
      this.reconnectAttempts = 0;
      this.reconnectInterval = 1000;
    };

    this.ws.onmessage = (event) => {
      try {
        console.log('Raw WebSocket message received:', event.data);
        const message: WebSocketMessage = JSON.parse(event.data);
        console.log('Parsed WebSocket message:', message);
        if (this.onMessage) {
          this.onMessage(message);
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onclose = (event) => {
      console.log('WebSocket connection closed:', event.code, event.reason);
      this.setStatus('disconnected');
      this.handleReconnect();
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.setStatus('error');
      this.ws?.close();
    };
  }

  public disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  private handleReconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => {
        this.reconnectAttempts++;
        this.connect();
      }, this.reconnectInterval);

      // Exponential backoff
      this.reconnectInterval *= 2;
    } else {
      console.error('Max reconnect attempts reached.');
    }
  }

  public subscribe(itemId: string): void {
    this.sendMessage({
      type: 'Subscribe',
      data: { item_id: itemId },
    });
  }

  public unsubscribe(itemId: string): void {
    this.sendMessage({
      type: 'Unsubscribe',
      data: { item_id: itemId },
    });
  }

  private sendMessage(data: any): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    } else {
      console.error('WebSocket is not connected.');
    }
  }

  private setStatus(status: ConnectionStatus): void {
    if (this.onStatusChange) {
      this.onStatusChange(status);
    }
  }
}

// Dynamically create the WebSocket URL to use the proxy
const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
const host = window.location.host;
const wsUrl = `${protocol}://${host}/ws`;

export const webSocketService = new WebSocketService(wsUrl);
