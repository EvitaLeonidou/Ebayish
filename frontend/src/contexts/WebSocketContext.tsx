import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { webSocketService } from '../services/WebSocketService';
import { WebSocketMessage } from '../types/websocket';
import { useAuth } from './AuthContext';

type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface WebSocketContextType {
  status: ConnectionStatus;
  lastMessage: WebSocketMessage | null;
  subscribe: (itemId: string) => void;
  unsubscribe: (itemId: string) => void;
}

const WebSocketContext = createContext<WebSocketContextType | undefined>(undefined);

export const WebSocketProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const { token } = useAuth();
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);

  useEffect(() => {
    console.log('WebSocketContext: Setting up with token:', token ? 'present' : 'absent');
    webSocketService.onStatusChange = setStatus;
    webSocketService.onMessage = setLastMessage;

    if (token) {
      console.log('WebSocketContext: Connecting with token');
      webSocketService.connect(token);
    } else {
      console.log('WebSocketContext: No token available, not connecting');
    }

    return () => {
      console.log('WebSocketContext: Disconnecting');
      webSocketService.disconnect();
    };
  }, [token]);

  const subscribe = (itemId: string) => {
    webSocketService.subscribe(itemId);
  };

  const unsubscribe = (itemId: string) => {
    webSocketService.unsubscribe(itemId);
  };

  return (
    <WebSocketContext.Provider value={{ status, lastMessage, subscribe, unsubscribe }}>
      {children}
    </WebSocketContext.Provider>
  );
};

export const useWebSocketContext = () => {
  const context = useContext(WebSocketContext);
  if (context === undefined) {
    throw new Error('useWebSocketContext must be used within a WebSocketProvider');
  }
  return context;
};
