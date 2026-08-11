import React from 'react';

const Welcome: React.FC = () => {
  return (
    <div className="flex min-h-[calc(100vh - 80px)] flex-col bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="flex flex-grow items-center justify-center px-4 py-5">
        <div className="grid w-full max-w-4xl grid-cols-1 overflow-hidden rounded-2xl shadow-2xl md:grid-cols-5">
          <div className="relative hidden bg-blue-600 md:col-span-2 md:block">
            <div className="absolute inset-0 bg-blue-600 bg-opacity-90">
              <div className="flex h-full flex-col items-center justify-center p-6 text-white">
                <h2 className="mb-6 text-2xl font-bold"> gamiesai </h2>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Welcome;
