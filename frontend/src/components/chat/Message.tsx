import React from 'react';
import { format } from 'date-fns';
import { useAuth } from '@/contexts/AuthContext';
import { Message as MessageType } from '@/types/chat';

interface MessageProps {
  message: MessageType;
}

const Message: React.FC<MessageProps> = ({ message }) => {
  const { user: currentUser } = useAuth();
  const isSender = message.user.id === currentUser?.id;

  // Helper to get the first initial from a username
  const getInitials = (username: string) => {
    return username ? username.charAt(0).toUpperCase() : '?';
  };

  return (
    <div className={`flex items-end gap-3 my-3 ${isSender ? 'justify-end' : ''}`}>
      {/* MODIFICATION: Display user initial in a colored circle */}
      {!isSender && (
        <div className="w-8 h-8 rounded-full bg-gray-300 flex items-center justify-center text-gray-600 font-semibold flex-shrink-0">
          {getInitials(message.user.username)}
        </div>
      )}
      <div
        className={`max-w-xs md:max-w-md p-3 rounded-2xl ${
          isSender
            ? 'bg-blue-600 text-white rounded-br-none'
            : 'bg-gray-200 text-gray-800 rounded-bl-none'
        }`}
      >
        <p className="text-sm">{message.content}</p>
        <p className={`text-xs mt-1 text-right ${isSender ? 'text-blue-200' : 'text-gray-500'}`}>
          {format(new Date(message.inserted_at), 'p')}
        </p>
      </div>
    </div>
  );
};

export default Message;
