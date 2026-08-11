import React from 'react';
import Header from '@/components/Header';

// The layout now accepts the page component as a `children` prop.
const UserLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <div className="min-h-screen bg-gray-50 w-full">
      <Header />
      <main className="max-w-7xl mx-auto py-8 px-4 sm:px-6 lg:px-8">
        {/* The page content is rendered here */}
        {children}
      </main>
    </div>
  );
};

export default UserLayout;
