import { createContext, useState, useEffect, useContext, ReactNode } from 'react';
import axios from 'axios';
import { useNavigate, useLocation } from 'react-router-dom';
import type { User } from '@/types/user';
import { jwtDecode } from 'jwt-decode'; // MODIFIED: Import the decoder

// --- MODIFICATION START ---
// Define an interface for the JWT payload to ensure type safety
interface JwtPayload {
  sub: string; // The user's ID (UUID)
  username: string;
  role: 'user' | 'admin';
  exp: number;
}
// --- MODIFICATION END ---

interface AuthContextType {
  isAuthenticated: boolean;
  user: User | null;
  token: string | null;
  isLoading: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
  setUser: React.Dispatch<React.SetStateAction<User | null>>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const checkAuthStatus = async () => {
      const storedToken = sessionStorage.getItem('auth_token');

      if (storedToken) {
        setToken(storedToken);
        axios.defaults.headers.common['Authorization'] = `Bearer ${storedToken}`;
        try {
          // --- MODIFICATION START ---
          // Decode the token to get user info directly, removing the need for an extra API call.
          const decodedToken = jwtDecode<JwtPayload>(storedToken);

          const userObject: User = {
            id: decodedToken.sub, // Use the 'sub' claim for the user ID
            username: decodedToken.username,
            role: decodedToken.role,
            // Fill in other fields with defaults as they aren't in the token
            email: '',
            first_name: '',
            last_name: '',
            phone: null,
            date_of_birth: null,
            status: 'confirmed',
            created_at: new Date().toISOString(),
          };
          setUser(userObject);
          // --- MODIFICATION END ---
        } catch (error) {
          console.error('Session token is invalid or expired. Logging out.');
          logout();
        }
      }
      setIsLoading(false);
    };

    checkAuthStatus();
  }, []);

  const login = async (username: string, password: string) => {
    const response = await axios.post<{ token: string }>('/api/login', {
      username,
      password,
    });

    if (response.data.token) {
      const authToken = response.data.token;
      setToken(authToken);
      sessionStorage.setItem('auth_token', authToken);
      axios.defaults.headers.common['Authorization'] = `Bearer ${authToken}`;

      // --- MODIFICATION START ---
      // Decode the token immediately after login to get the full user details
      const decodedToken = jwtDecode<JwtPayload>(authToken);

      const currentUser: User = {
        id: decodedToken.sub, // Use the 'sub' claim for the user ID
        username: decodedToken.username,
        role: decodedToken.role,
        // Fill in other fields with defaults
        email: '',
        first_name: '',
        last_name: '',
        phone: null,
        date_of_birth: null,
        status: 'confirmed',
        created_at: new Date().toISOString(),
      };
      setUser(currentUser);
      sessionStorage.setItem('username', currentUser.username); // Store the canonical username
      // --- MODIFICATION END ---

      const from = location.state?.from?.pathname || null;
      if (currentUser.role === 'admin') {
        navigate(from && from.startsWith('/admin') ? from : '/admin');
      } else {
        navigate(from || '/user/profile');
      }
    }
  };

  const logout = () => {
    setUser(null);
    setToken(null);
    sessionStorage.removeItem('auth_token');
    sessionStorage.removeItem('username');
    delete axios.defaults.headers.common['Authorization'];
    navigate('/');
  };

  const value = {
    isAuthenticated: !isLoading && !!user,
    user,
    token,
    isLoading,
    login,
    logout,
    setUser,
  };

  return <AuthContext.Provider value={value}>{!isLoading && children}</AuthContext.Provider>;
}

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
