export interface User {
  id: string; // Assuming UUID from backend
  username: string;
  first_name: string;
  last_name: string;
  email: string;
  phone: string | null;
  date_of_birth: string | null;
  role: 'user' | 'admin';
  status: 'pending' | 'confirmed' | 'suspended'; // Expanded statuses
  created_at: string; // ISO 8601 date string
  last_login?: string;
}
