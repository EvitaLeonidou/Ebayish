import React, { useState, useEffect, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { getConnections } from '@/services/connectionService';
import { createOrGetChatRoom } from '@/services/chatService';
import { Conversation } from '@/types/chat';
import ConversationList from '@/components/chat/ConversationList';
import ChatWindow from '@/components/chat/ChatWindow';
import { MessageSquare, Loader2 } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
// import { useAuth } from '@/contexts/AuthContext';

const Messaging: React.FC = () => {
  const { userId } = useParams<{ userId?: string }>();
  const navigate = useNavigate();
  //   const { user } = useAuth();
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeChatRoomId, setActiveChatRoomId] = useState<string | null>(null);
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let isMounted = true;
    const fetchInitialData = async () => {
      try {
        const connectionsData = await getConnections();
        if (!isMounted) return;

        const convos = connectionsData.map((c) => ({
          id: c.connected_user.id,
          username: c.connected_user.username,
        }));
        setConversations(convos);

        // If a userId is in the URL and that user is a valid connection, select them.
        const userFromParam = convos.find((c) => c.id === userId);
        if (userFromParam) {
          handleSelectConversation(userFromParam.id);
        }
      } catch (error) {
        console.error('Failed to fetch connections:', error);
      } finally {
        if (isMounted) setIsLoading(false);
      }
    };

    fetchInitialData();
    return () => {
      isMounted = false;
    };
  }, [userId]); // Re-run if the userId in the URL changes

  const handleSelectConversation = async (otherUserId: string) => {
    // Avoid re-fetching if the conversation is already active
    if (activeConversationId === otherUserId) return;

    try {
      // Get the chat room ID from the backend
      const roomId = await createOrGetChatRoom(otherUserId);
      setActiveConversationId(otherUserId);
      setActiveChatRoomId(roomId);

      // Update the URL without a full page reload for a smoother experience
      navigate(`/messaging/${otherUserId}`, { replace: true });
    } catch (error) {
      console.error('Failed to create or get chat room:', error);
    }
  };

  const activeUser = useMemo(
    () => conversations.find((c) => c.id === activeConversationId),
    [conversations, activeConversationId]
  );

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center p-8">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 md:p-6">
      <h1 className="text-4xl font-bold text-gray-900 mb-6">Your Messages</h1>
      <Card className="h-[calc(100vh-220px)] w-full shadow-lg">
        <CardContent className="p-0 h-full">
          <div className="grid grid-cols-1 md:grid-cols-4 h-full">
            {/* Conversation List */}
            <div className="md:col-span-1 h-full overflow-y-auto border-r">
              <ConversationList
                conversations={conversations}
                activeConversationId={activeConversationId}
                onSelectConversation={handleSelectConversation}
              />
            </div>
            {/* Chat Window */}
            <div className="md:col-span-3 h-full hidden md:flex flex-col overflow-hidden">
              {activeChatRoomId && activeUser ? (
                <ChatWindow
                  chatRoomId={activeChatRoomId}
                  otherUser={{ username: activeUser.username }}
                />
              ) : (
                <div className="flex flex-col items-center justify-center h-full text-center text-gray-500 bg-gray-50">
                  <MessageSquare className="w-16 h-16 mb-4" />
                  <h2 className="text-xl font-semibold">Select a conversation</h2>
                  <p>Choose someone from the list to start chatting.</p>
                </div>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default Messaging;
