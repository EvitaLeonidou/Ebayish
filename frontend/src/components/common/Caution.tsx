import React from 'react';

const Caution: React.FC<{ className?: string }> = ({ className }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="32"
    height="32"
    viewBox="0 0 48 48"
    className={className || 'h-5 w-5'}
  >
    <g fill="yellow" stroke="black" stroke-width="4">
      <path stroke-linejoin="round" d="M24 5L2 43h44z" clip-rule="evenodd" />
      <path stroke-linecap="round" d="M24 35v1m0-17l.008 10" />
    </g>
  </svg>
);

export default Caution;
