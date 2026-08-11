import { Button } from '@/components/ui/button';
import { FileQuestion } from 'lucide-react';
import React from 'react';
import { Link } from 'react-router-dom';

const NotFound: React.FC = () => {
  return (
    <div className="flex h-screen w-full flex-col items-center justify-center bg-gray-50 p-4">
      <div className="w-full max-w-md text-center">
        <div className="flex justify-center">
          <FileQuestion className="h-24 w-24 text-blue-400" />
        </div>
        <h1 className="mt-6 text-6xl font-bold tracking-tight text-gray-800 sm:text-7xl">404</h1>
        <h2 className="mt-4 text-2xl font-semibold text-gray-900">Page Not Found</h2>
        <p className="mt-2 text-base text-gray-600">
          Sorry, we couldn't find the page you were looking for. It might have been moved or
          deleted.
        </p>
        <Link to="/" className="mt-8 inline-block">
          <Button className="h-12 px-8 text-lg bg-blue-600 hover:bg-blue-700">
            Return to Homepage
          </Button>
        </Link>
      </div>
    </div>
  );
};

export default NotFound;
