import { authFetch } from '@/utils/auth-fetch';

/**
 * Represents the structure of a user connection from the backend.
 */
interface Connection {
  connected_user: {
    id: string;
    username: string;
  };
}

/**
 * Fetches the list of users the current user can message.
 * In a real-world scenario, this might be users you've bought from, sold to, or have ongoing bids with.
 * For this implementation, we'll assume an endpoint that provides this list.
 */
export const getConnections = async (): Promise<Connection[]> => {
  try {
    // This endpoint will need to be created in your backend.
    // It should return a list of users that the authenticated user has a "connection" with.
    const response = await authFetch('/api/users/connections');
    if (!response.ok) {
      throw new Error('Failed to fetch connections');
    }
    return await response.json();
  } catch (error) {
    console.error('Error fetching connections:', error);
    // Return an empty array as a fallback to prevent the UI from crashing.
    return [];
  }
};
