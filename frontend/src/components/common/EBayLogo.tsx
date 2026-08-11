import React from 'react';
import { useNavigate } from 'react-router-dom';

const EbayLogo: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="flex items-center cursor-pointer" onClick={() => navigate('/')}>
      <h1 className="text-3xl font-bold tracking-tighter">
        <span className="text-ebay-red">e</span>
        <span className="text-ebay-blue">B</span>
        <span className="text-ebay-yellow">a</span>
        <span className="text-ebay-green">y</span>
        <span className="text-gray-700">ish</span>
      </h1>
    </div>
  );
};

export default EbayLogo;
