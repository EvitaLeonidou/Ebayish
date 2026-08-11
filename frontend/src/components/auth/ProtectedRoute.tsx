import NotAuthorized from './NotAuthorized';
import { useAuth } from '@/contexts/AuthContext';
import { Loader2 } from 'lucide-react';
import React from 'react';
import { Navigate, useLocation } from 'react-router-dom';

type ProtectedRouteProps = {
  children: React.ReactNode;
  allowedRoles: Array<'user' | 'admin'>;
};

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children, allowedRoles }) => {
  const { user, isLoading } = useAuth();
  const location = useLocation();

  if (isLoading) {
    return (
      <div className="flex h-screen w-full items-center justify-center">
        <div className="text-center">
          <Loader2 className="mx-auto h-12 w-12 animate-spin text-blue-600" />
          <p className="mt-4 text-lg text-gray-600">Verifying access...</p>
        </div>
      </div>
    );
  }

  if (!user) {
    return <Navigate to="/" state={{ from: location }} replace />;
  }

  if (!user.role || !allowedRoles.includes(user.role as 'user' | 'admin')) {
    return <NotAuthorized />;
  }

  return <>{children}</>;
};

export default ProtectedRoute;
