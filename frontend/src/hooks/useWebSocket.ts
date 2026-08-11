import { useEffect, useState } from 'react';
import { useWebSocketContext } from '../contexts/WebSocketContext';
import { AuctionEvent } from '../types/websocket';

export const useWebSocket = (itemId: string) => {
  const { status, lastMessage, subscribe, unsubscribe } = useWebSocketContext();
  const [eventData, setEventData] = useState<any>(null);
  const [lastEventType, setLastEventType] = useState<AuctionEvent | null>(null);

  useEffect(() => {
    if (status === 'connected') {
      subscribe(itemId);
    }

    return () => {
      if (status === 'connected') {
        unsubscribe(itemId);
      }
    };
  }, [status, itemId, subscribe, unsubscribe]);

  useEffect(() => {
    if (lastMessage && lastMessage.payload.itemId === itemId) {
      setLastEventType(lastMessage.type);
      setEventData(lastMessage.payload);
    }
  }, [lastMessage, itemId]);

  return { status, lastEventType, eventData };
};
