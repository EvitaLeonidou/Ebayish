import React, { useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Users, Package, Home, Menu, X, LogOut, ListTree, ShoppingCart } from 'lucide-react';

const AdminLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const navigate = useNavigate();
  const location = useLocation();
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const menuItems = [
    { icon: Home, label: 'Dashboard', path: '/admin' },
    { icon: Users, label: 'Users', path: '/admin/users' },
    { icon: Package, label: 'Listings', path: '/admin/listings' },
    { icon: ShoppingCart, label: 'Items Sold', path: '/admin/items-sold' },
    { icon: ListTree, label: 'Categories', path: '/admin/categories' },
  ];

  const isActive = (path: string) => {
    // Exact match for the dashboard route
    if (path === '/admin' && location.pathname === '/admin') return true;
    // StartsWith for all other admin routes
    return path !== '/admin' && location.pathname.startsWith(path);
  };

  return (
    <div className="min-h-screen bg-gray-50 flex">
      <div
        className={`fixed inset-y-0 left-0 z-50 w-64 bg-white shadow-lg transform transition-transform duration-300 ease-in-out lg:translate-x-0 lg:static lg:inset-0 ${
          sidebarOpen ? 'translate-x-0' : '-translate-x-full'
        }`}
      >
        <div className="flex items-center justify-between h-16 px-6 border-b">
          <h1 className="text-xl font-bold text-ebay-blue">Admin Panel</h1>
          <Button
            variant="outline"
            size="sm"
            className="lg:hidden hover:bg-slate-100 bg-white border-0 bg-transparent"
            onClick={() => setSidebarOpen(false)}
          >
            <X className="h-5 w-5" />
          </Button>
        </div>
        <nav className="mt-6 px-3">
          <div className="space-y-1">
            {menuItems.map((item) => (
              <Button
                key={item.path}
                variant={isActive(item.path) ? 'default' : 'outline'}
                className={`w-full justify-start gap-3 ${
                  isActive(item.path)
                    ? 'bg-blue-700 text-white hover:bg-blue-700'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
                onClick={() => {
                  navigate(item.path);
                  setSidebarOpen(false);
                }}
              >
                <item.icon className="h-5 w-5" />
                {item.label}
              </Button>
            ))}
          </div>
          <div className="mt-8 pt-6 border-t border-gray-200">
            <Button
              variant="outline"
              className="w-full justify-start gap-3 text-gray-700 bg-white hover:bg-gray-100"
              onClick={() => navigate('/')}
            >
              <LogOut className="h-5 w-5" />
              Back to Site
            </Button>
          </div>
        </nav>
      </div>
      <div className="flex-1 flex flex-col min-w-0">
        <header className="bg-white shadow-sm border-b h-16 flex items-center px-6 lg:px-8">
          <Button
            variant="outline"
            size="sm"
            className="lg:hidden mr-4 hover:ring-1 ring-gray-300"
            onClick={() => setSidebarOpen(true)}
          >
            <Menu className="h-5 w-5" />
          </Button>
          <div className="flex-1">
            <h2 className="text-lg font-semibold text-gray-900">
              {menuItems.find((item) => isActive(item.path))?.label || 'Admin'}
            </h2>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-600">Welcome, Admin</span>
            <div className="w-8 h-8 bg-ebay-blue rounded-full flex items-center justify-center">
              <span className="text-white text-sm font-semibold">A</span>
            </div>
          </div>
        </header>
        <main className="flex-1 overflow-auto p-6 lg:p-8">{children}</main>
      </div>
    </div>
  );
};

export default AdminLayout;
