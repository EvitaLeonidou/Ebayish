import { authFetch } from '@/utils/auth-fetch';
import { Message } from '@/types/chat';

const API_BASE_URL = '/api/chat';

/**
 * Creates a new chat room with another user or retrieves the existing one.
 * The backend should handle the logic of finding an existing room or creating a new one.
 * @param otherUserId - The ID of the user to start a conversation with.
 * @returns The ID of the chat room.
 */
export const createOrGetChatRoom = async (otherUserId: string): Promise<string> => {
  try {
    const response = await authFetch(`${API_BASE_URL}/rooms`, {
      method: 'POST',
      body: JSON.stringify({ other_user_id: otherUserId }),
    });

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.message || 'Failed to create or get chat room');
    }

    const data = await response.json();
    if (!data.room_id) {
      throw new Error('Invalid response from server: room_id missing');
    }

    return data.room_id;
  } catch (error) {
    console.error('Error in createOrGetChatRoom:', error);
    throw error;
  }
};

/**
 * Fetches the message history for a specific chat room.
 * @param chatRoomId - The ID of the chat room.
 * @returns A promise that resolves to an array of messages.
 */
export const getMessageHistory = async (chatRoomId: string): Promise<Message[]> => {
  try {
    const response = await authFetch(`${API_BASE_URL}/rooms/${chatRoomId}/messages`);

    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.message || 'Failed to fetch message history');
    }

    const data = await response.json();
    // Assuming the backend returns messages oldest to newest.
    // No sorting is needed if the backend already provides it in the correct order.
    return data.messages || [];
  } catch (error) {
    console.error('Error fetching message history:', error);
    throw error;
  }
};
