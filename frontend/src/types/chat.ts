/**
 * Represents a conversation partner in the conversation list.
 */
export interface Conversation {
  id: string; // The other user's ID
  username: string;
}

/**
 * Represents a single message within a chat window.
 */
export interface Message {
  id: string; // Can be a temporary ID for optimistic updates
  content: string | null;
  inserted_at: string; // ISO 8601 date string
  user: {
    id: string;
    username: string;
  };
}
