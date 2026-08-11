// frontend/src/App.tsx

import ProtectedRoute from './components/auth/ProtectedRoute';
import routes from './routes/routes';
import { Route, Routes } from 'react-router-dom';
import { AuthProvider } from './contexts/AuthContext';
import { CartProvider } from './contexts/CartContext';
import { Toaster } from 'sonner';
import { WebSocketProvider } from './contexts/WebSocketContext'; // ADD THIS LINE

const AppContent: React.FC = () => {
  // This component now handles nested routes from the configuration file.
  return (
    <Routes>
      {routes.map((route, index) => (
        <Route
          key={index}
          path={route.path}
          element={
            route.protected ? (
              <ProtectedRoute allowedRoles={route.roles!}>{route.element}</ProtectedRoute>
            ) : (
              route.element
            )
          }
        />
      ))}
    </Routes>
  );
};

const App: React.FC = () => {
  return (
    <AuthProvider>
      <CartProvider>
        {/* WRAP YOUR APP CONTENT WITH THE WEBSOCKET PROVIDER */}
        <WebSocketProvider>
          <Toaster
            position="top-right"
            richColors
            expand={true}
            visibleToasts={5}
            closeButton={true}
          />
          <AppContent />
        </WebSocketProvider>
      </CartProvider>
    </AuthProvider>
  );
};

export default App;
