import React from 'react';

const Roblox: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="32"
    height="32"
    viewBox="0 0 24 24"
    className={className || 'h-5 w-5'}
  >
    <path
      fill="currentColor"
      d="M18.926 23.998L0 18.892L5.075.002L24 5.108ZM15.348 10.09l-5.282-1.453l-1.414 5.273l5.282 1.453z"
    />
  </svg>
);

export default Roblox;
