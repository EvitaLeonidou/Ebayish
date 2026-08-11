/**
 * Utility function for making authenticated fetch requests
 * Automatically adds Authorization header when user is logged in
 */

export const authFetch = async (url: string, options: RequestInit = {}): Promise<Response> => {
  const token = sessionStorage.getItem('auth_token');
  console.log('authFetch called for:', url);
  console.log('Token available:', !!token);

  const headers = {
    'Content-Type': 'application/json',
    ...options.headers,
  };

  if (token) {
    (headers as any)['Authorization'] = `Bearer ${token}`;
    console.log('Authorization header set');
  } else {
    console.warn('No token available for authenticated request');
  }

  return fetch(url, {
    ...options,
    headers,
  });
};
