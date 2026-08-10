import {
  createContext,
  useContext,
  useEffect,
  useState,
  ReactNode
} from 'react';
import { request } from '../utils/fetchUtils';
import { useTauriListener } from './useTauriListener';

interface UserResponse {
  data: {
    profile: {
      username: string;
    };
  };
}

interface AuthContextType {
  username: string | null;
  fetchUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider = ({ children }: { children: ReactNode }) => {
  const [username, setUsername] = useState<string | null>(null);

  const fetchUser = async () => {
    const resp = await request('/api/auth/checkAuth', 'GET');

    if (resp.ok) {
      const json = await resp.json();
      setUsername((json as UserResponse)?.data?.profile?.username);
    } else {
      setUsername(null);
    }
  };

  useEffect(() => {
    fetchUser();
  }, []);

  useTauriListener('logout', fetchUser);

  return (
    <AuthContext.Provider value={{ username, fetchUser }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error('useAuth must be used inside an AuthProvider');
  }
  return ctx;
};
