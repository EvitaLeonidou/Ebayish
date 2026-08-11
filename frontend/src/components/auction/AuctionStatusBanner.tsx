import React from 'react';
import { Item } from '@/types/item';
import { CheckCircle, Clock, XCircle, ShieldQuestion } from 'lucide-react';

interface AuctionStatusBannerProps {
  item: Item;
}

const AuctionStatusBanner: React.FC<AuctionStatusBannerProps> = ({ item }) => {
  // const isEnded = item.status === 'ended' || item.status === 'sold';

  let bgColor = 'bg-gray-100';
  let textColor = 'text-gray-800';
  let Icon = ShieldQuestion;
  let message = 'Status unknown.';

  switch (item.status) {
    case 'active':
      bgColor = 'bg-green-100';
      textColor = 'text-green-800';
      Icon = Clock;
      message = 'This auction is currently active. Place your bids!';
      break;
    case 'ended':
      bgColor = 'bg-red-100';
      textColor = 'text-red-800';
      Icon = XCircle;
      message = `This auction has ended. The winning bid was $${Number(item.current_price || 0).toFixed(2)}.`;
      if (item.winner) {
        message += ` Congratulations to the winner, ${item.winner.username}!`;
      }
      break;
    case 'sold':
      bgColor = 'bg-blue-100';
      textColor = 'text-blue-800';
      Icon = CheckCircle;
      message = 'This item has been sold and is no longer available.';
      break;
    case 'pending':
      bgColor = 'bg-yellow-100';
      textColor = 'text-yellow-800';
      Icon = ShieldQuestion;
      message = 'This listing is pending approval and is not yet active.';
      break;
  }

  return (
    <div
      className={`p-4 rounded-lg flex items-center transition-all duration-300 ${bgColor} ${textColor}`}
    >
      <Icon className="h-6 w-6 mr-3 flex-shrink-0" />
      <p className="text-sm font-semibold">{message}</p>
    </div>
  );
};

export default AuctionStatusBanner;
