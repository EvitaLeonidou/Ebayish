import React, { useState, useEffect, useRef } from 'react';
import { useAuth } from '@/contexts/AuthContext';
import { getMessageHistory } from '@/services/chatService';
import { Message as MessageType } from '@/types/chat';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Send, User, Loader2, Trash2 } from 'lucide-react';
import { format } from 'date-fns';
import TypingIndicator from './TypingIndicator';
import { toast } from 'sonner';
import { webSocketService } from '@/services/WebSocketService';
import { AuctionEvent, NewMessagePayload, MessageDeletedPayload } from '@/types/websocket';
import { authFetch } from '@/utils/auth-fetch';

interface ChatWindowProps {
  chatRoomId: string;
  otherUser: { username: string };
}

const ChatWindow: React.FC<ChatWindowProps> = ({ chatRoomId, otherUser }) => {
  const { user, token } = useAuth();
  const [messages, setMessages] = useState<MessageType[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isTyping, setIsTyping] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    setIsLoading(true);
    setMessages([]);

    if (!chatRoomId || !token) {
      setIsLoading(false);
      return;
    }

    // Load message history
    getMessageHistory(chatRoomId)
      .then((history) => setMessages(history))
      .catch((err) => {
        console.error('Failed to get message history', err);
        toast.error('Could not load message history.');
      })
      .finally(() => setIsLoading(false));

    // Connect to WebSocket if not already connected
    if (!webSocketService || !token) {
      toast.error('Failed to connect to real-time messaging.');
      return;
    }

    webSocketService.connect(token);

    // Set up message handler for this chat room
    const handleWebSocketMessage = (message: any) => {
      if (message.type === AuctionEvent.NewMessage) {
        const payload = message.data as NewMessagePayload;
        // Only process messages for this chat room
        if (payload.chat_room_id === chatRoomId) {
          setMessages((prev) => {
            // Check if this message already exists to prevent duplicates
            // Look for both the message ID and also check for same content/timestamp to catch optimistic updates
            const existingMessage = prev.find(
              (m) =>
                m.id === payload.message_id ||
                (m.content === payload.content &&
                  m.user.username === payload.sender_username &&
                  Math.abs(
                    new Date(m.inserted_at).getTime() - new Date(payload.timestamp).getTime()
                  ) < 5000) // Within 5 seconds
            );

            if (existingMessage) {
              // If we found a match but it's a temp ID, replace it with the real message
              if (existingMessage.id.startsWith('temp-')) {
                return prev.map((m) =>
                  m.id === existingMessage.id
                    ? {
                        id: payload.message_id,
                        content: payload.content,
                        inserted_at: payload.timestamp,
                        user: {
                          id: payload.sender_username === user?.username ? user.id : 'other',
                          username: payload.sender_username,
                        },
                      }
                    : m
                );
              }
              // Otherwise, it's a real duplicate, so don't add it
              return prev;
            }

            return [
              ...prev,
              {
                id: payload.message_id,
                content: payload.content,
                inserted_at: payload.timestamp,
                user: {
                  id: payload.sender_username === user?.username ? user.id : 'other',
                  username: payload.sender_username,
                },
              },
            ];
          });
          setIsTyping(false);
        }
      } else if (message.type === AuctionEvent.MessageDeleted) {
        const payload = message.data as MessageDeletedPayload;
        // Only process deletions for this chat room
        if (payload.chat_room_id === chatRoomId) {
          setMessages((prev) => prev.filter((m) => m.id !== payload.message_id));
        }
      }
    };

    // Store the previous handler to restore it later
    const previousHandler = webSocketService.onMessage;
    webSocketService.onMessage = handleWebSocketMessage;

    return () => {
      if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
      // Restore the previous handler
      webSocketService.onMessage = previousHandler;
    };
  }, [chatRoomId, token, user?.id, user?.username]);

  useEffect(() => {
    // Scroll to the bottom whenever new messages are added.
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newMessage.trim()) return;

    setIsSending(true);
    const messageContent = newMessage.trim();

    // Optimistic update: add the message to the UI immediately.
    const tempId = `temp-${Date.now()}`;
    const optimisticMessage: MessageType = {
      id: tempId,
      content: messageContent,
      inserted_at: new Date().toISOString(),
      user: {
        id: user!.id,
        username: user!.username,
      },
    };
    setMessages((prev) => [...prev, optimisticMessage]);
    setNewMessage('');

    try {
      const response = await authFetch(`/api/chat/rooms/${chatRoomId}/messages`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ content: messageContent }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to send message');
      }

      const realMessage = await response.json();

      // Replace optimistic message with real one
      setMessages((prev) => prev.map((m) => (m.id === tempId ? realMessage : m)));
    } catch (error) {
      console.error('Failed to send message:', error);
      // If sending fails, remove the optimistic message.
      setMessages((prev) => prev.filter((m) => m.id !== tempId));
      setNewMessage(messageContent); // Restore the input
      toast.error('Failed to send message. Please try again.');
    } finally {
      setIsSending(false);
    }
  };

  const handleTyping = (e: React.ChangeEvent<HTMLInputElement>) => {
    setNewMessage(e.target.value);
    // Note: Typing indicators removed for simplicity
  };

  const handleDeleteMessage = async (messageId: string) => {
    try {
      // Optimistically remove the message for the sender immediately
      setMessages((prev) => prev.filter((m) => m.id !== messageId));

      const response = await authFetch(`/api/chat/rooms/${chatRoomId}/messages/${messageId}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to delete message');
      }

      toast.success('Message deleted');
    } catch (error) {
      console.error('Failed to delete message:', error);
      // If deletion failed, restore the message
      // We'd need to refetch messages or store the original message to restore it properly
      toast.error('Failed to delete message. Please try again.');
      // For now, just reload the messages
      window.location.reload();
    }
  };

  if (isLoading) {
    return (
      <div className="flex-grow flex items-center justify-center h-full">
        <Loader2 className="h-10 w-10 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-white border-l">
      <header className="p-4 border-b">
        <h2 className="text-xl font-bold">{otherUser.username}</h2>
      </header>

      <div className="flex-1 min-h-0 p-4 overflow-y-auto">
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex items-end gap-3 my-3 group relative ${
              msg.user.id === user?.id ? 'justify-end' : ''
            }`}
          >
            {msg.user.id !== user?.id && <User className="w-8 h-8 text-gray-400" />}
            <div className="relative">
              <div
                className={`max-w-xs md:max-w-md p-3 rounded-2xl ${
                  msg.user.id === user?.id
                    ? 'bg-blue-600 text-white rounded-br-none'
                    : 'bg-gray-200 text-gray-800 rounded-bl-none'
                }`}
              >
                <p className="text-sm">{msg.content}</p>
                <p
                  className={`text-xs mt-1 text-right ${
                    msg.user.id === user?.id ? 'text-blue-200' : 'text-gray-500'
                  }`}
                >
                  {(() => {
                    try {
                      // Handle both ISO strings and nanosecond precision timestamps
                      let dateStr = msg.inserted_at;
                      if (typeof dateStr === 'string' && dateStr.includes('.')) {
                        // Truncate nanoseconds to milliseconds (keep only 3 digits after decimal)
                        dateStr = dateStr.replace(/(\.\d{3})\d*/, '$1');
                      }
                      return format(new Date(dateStr), 'p');
                    } catch (error) {
                      // Fallback to current time without logging
                      return format(new Date(), 'p');
                    }
                  })()}
                </p>
              </div>
              {/* Delete button - only show for user's own messages */}
              {msg.user.id === user?.id && (
                <button
                  onClick={() => handleDeleteMessage(msg.id)}
                  className="absolute -top-2 -right-2 opacity-0 group-hover:opacity-100 transition-opacity bg-red-500 hover:bg-red-600 text-white rounded-full p-1.5 shadow-md"
                  title="Delete message"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              )}
            </div>
          </div>
        ))}
        {isTyping && (
          <div className="flex items-end gap-3 my-3">
            <User className="w-8 h-8 text-gray-400" />
            <TypingIndicator />
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <footer className="p-4 border-t bg-white">
        <form onSubmit={handleSendMessage} className="flex items-center gap-3">
          <Input
            type="text"
            placeholder="Type a message..."
            value={newMessage}
            onChange={handleTyping}
            className="flex-grow"
            autoComplete="off"
            disabled={isSending}
          />
          <Button type="submit" size="icon" disabled={isSending || !newMessage.trim()}>
            {isSending ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Send className="w-5 h-5" />
            )}
          </Button>
        </form>
      </footer>
    </div>
  );
};

export default ChatWindow;
