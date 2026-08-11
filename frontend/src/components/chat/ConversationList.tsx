import React from 'react';
import { Conversation } from '@/types/chat';
import { User } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils'; // Assuming you have a utility for merging class names. If not, you can install `clsx` and `tailwind-merge`.

interface ConversationListProps {
  conversations: Conversation[];
  activeConversationId: string | null;
  onSelectConversation: (userId: string) => void;
}

const ConversationList: React.FC<ConversationListProps> = ({
  conversations,
  activeConversationId,
  onSelectConversation,
}) => {
  return (
    <Card className="h-full flex flex-col rounded-none border-t-0 border-b-0 border-l-0 md:rounded-lg md:border">
      <CardHeader>
        <CardTitle>Messages</CardTitle>
      </CardHeader>
      <CardContent className="flex-grow overflow-y-auto p-2">
        {conversations.length > 0 ? (
          <ul className="space-y-1">
            {conversations.map((convo) => (
              <li key={convo.id}>
                <button
                  onClick={() => onSelectConversation(convo.id)}
                  className={cn(
                    'flex items-center gap-4 p-3 rounded-lg transition-colors w-full text-left bg-white',
                    activeConversationId === convo.id
                      ? 'bg-blue-100 text-blue-900'
                      : 'hover:bg-gray-100 text-gray-900'
                  )}
                >
                  <div className="relative flex-shrink-0 bg-gray-200 rounded-full p-2">
                    <User className="w-8 h-8 text-gray-500" />
                  </div>
                  <div>
                    <p className="font-semibold">{convo.username}</p>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-center text-gray-500 pt-8">No conversations yet.</p>
        )}
      </CardContent>
    </Card>
  );
};

export default ConversationList;
