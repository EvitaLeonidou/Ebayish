import React from 'react';

import UserLayout from '@/components/layout/UserLayout';
import AdminLayout from '@/components/layout/AdminLayout';

import Landing from '@/pages/Landing';
import Login from '@/pages/Login';
import Signup from '@/pages/Signup';
import Profile from '@/pages/Profile';
import AdminDashboard from '@/pages/admin/AdminDashboard';
import UserManagement from '@/pages/admin/UserManagement';
import ListingManagement from '@/pages/admin/ListingManagement';
import ItemsSold from '@/pages/admin/ItemsSold';
import NotFound from '@/pages/NotFound';
import CreateListing from '@/pages/CreateListing';
import Marketplace from '@/pages/Marketplace';
import ItemDetail from '@/pages/ItemDetail';
import EditListing from '@/pages/EditListing';
import MyBids from '@/pages/MyBids';
import CategoryManagement from '@/pages/admin/CategoryManagement';
import Cart from '@/pages/Cart';
import Help from '@/pages/Help';
import Messaging from '@/pages/Messaging';
import Notifications from '@/pages/Notifications';

export type RouteConfig = {
  path: string;
  element: React.ReactNode;
  protected?: boolean;
  roles?: Array<'user' | 'admin'>;
};

const routes: RouteConfig[] = [
  // --- Public Routes ---
  {
    path: '/',
    element: <Landing />,
  },
  {
    path: '/login',
    element: <Login />,
  },
  {
    path: '/signup',
    element: <Signup />,
  },
  {
    path: '/marketplace',
    element: (
      <UserLayout>
        <Marketplace />
      </UserLayout>
    ),
  },
  {
    path: '/item/:itemId',
    element: (
      <UserLayout>
        <ItemDetail />
      </UserLayout>
    ),
  },
  {
    path: '/help',
    element: (
      <UserLayout>
        <Help />
      </UserLayout>
    ),
  },

  // --- Protected User Routes ---
  {
    path: '/user/profile',
    element: (
      <UserLayout>
        <Profile />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },
  {
    path: '/messaging',
    element: (
      <UserLayout>
        <Messaging />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },
  {
    path: '/messaging/:userId',
    element: (
      <UserLayout>
        <Messaging />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },
  {
    path: '/user/bids',
    element: (
      <UserLayout>
        <MyBids />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },
  {
    path: '/notifications',
    element: (
      <UserLayout>
        <Notifications />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },
  {
    path: '/cart',
    element: (
      <UserLayout>
        <Cart />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },
  {
    path: '/sell',
    element: (
      <UserLayout>
        <CreateListing />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },
  {
    path: '/sell/edit/:itemId',
    element: (
      <UserLayout>
        <EditListing />
      </UserLayout>
    ),
    protected: true,
    roles: ['user', 'admin'],
  },

  // --- Protected Admin Routes ---
  {
    path: '/admin',
    element: (
      <AdminLayout>
        <AdminDashboard />
      </AdminLayout>
    ),
    protected: true,
    roles: ['admin'],
  },
  {
    path: '/admin/users',
    element: (
      <AdminLayout>
        <UserManagement />
      </AdminLayout>
    ),
    protected: true,
    roles: ['admin'],
  },
  {
    path: '/admin/listings',
    element: (
      <AdminLayout>
        <ListingManagement />
      </AdminLayout>
    ),
    protected: true,
    roles: ['admin'],
  },
  {
    path: '/admin/items-sold',
    element: (
      <AdminLayout>
        <ItemsSold />
      </AdminLayout>
    ),
    protected: true,
    roles: ['admin'],
  },
  {
    path: '/admin/categories',
    element: (
      <AdminLayout>
        <CategoryManagement />
      </AdminLayout>
    ),
    protected: true,
    roles: ['admin'],
  },

  // --- Not Found Route ---
  {
    path: '*',
    element: <NotFound />,
  },
];

export default routes;
