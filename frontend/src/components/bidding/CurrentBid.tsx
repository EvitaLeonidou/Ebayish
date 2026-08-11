import React from 'react';
import { Clock, Users, ArrowUp } from 'lucide-react';
import { useCountdown } from '@/hooks/useCountdown';
import { Item } from '@/types/item';

interface CurrentBidProps {
  item: Item;
  minNextBid: number; // Receive minNextBid as a prop
  timeLeft?: number;
}

const formatPrice = (price: number) => {
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
};

const CountdownTimer: React.FC<{ endTime: string; timeLeft?: number }> = ({
  endTime,
  timeLeft,
}) => {
  const { days, hours, minutes, seconds, isFinished } = useCountdown(endTime, timeLeft);

  if (isFinished) {
    return <span className="font-bold text-red-600">Auction Ended</span>;
  }

  const isUnderAnHour = days === 0 && hours === 0;
  const isUnderTenMinutes = isUnderAnHour && minutes < 10;
  const urgencyClass = isUnderTenMinutes
    ? 'text-red-600'
    : isUnderAnHour
      ? 'text-yellow-600'
      : 'text-gray-900';

  return (
    <span className={`font-bold ${urgencyClass}`}>
      {days > 0 && `${days}d `}
      {hours > 0 && `${hours}h `}
      {minutes > 0 && `${minutes}m `}
      {seconds}s
    </span>
  );
};

const CurrentBid: React.FC<CurrentBidProps> = ({ item, minNextBid, timeLeft }) => {
  const bidCount = Number(item.number_of_bids ?? 0);
  const currentPrice = (() => {
    // For auction items, always use 'currently' if it exists, as that's what backend validates against
    if (item.listing_type === 'auction') {
      // Use currently if it exists and is higher than starting price, otherwise use price
      const priceValue = Number(item.price || 0);
      const currentlyValue = Number(item.currently || 0);

      // Backend sets 'currently' as the actual current bid price, regardless of bid count
      return currentlyValue > 0 ? currentlyValue : priceValue;
    }
    // For fixed price items, always use price
    return Number(item.price || 0);
  })();

  return (
    <div className="space-y-4">
      <div>
        <p className="text-sm text-gray-600">{bidCount === 0 ? 'Starting Price' : 'Current Bid'}</p>
        <p className="text-3xl font-bold text-gray-900">{formatPrice(currentPrice)}</p>
      </div>
      <div className="grid grid-cols-2 gap-4 text-sm">
        <div className="flex items-center">
          <Clock className="h-4 w-4 mr-2 text-gray-500" />
          <div>
            <p className="text-gray-600">Time Left</p>
            {item.ends ? (
              <CountdownTimer endTime={item.ends} timeLeft={timeLeft} />
            ) : (
              <p className="font-bold">N/A</p>
            )}
          </div>
        </div>
        <div className="flex items-center">
          <Users className="h-4 w-4 mr-2 text-gray-500" />
          <div>
            <p className="text-gray-600">Bids</p>
            <p className="font-bold">{item.number_of_bids || 0}</p>
          </div>
        </div>
      </div>
      {item.status === 'active' && (
        <div className="flex items-center text-sm bg-blue-50 p-3 rounded-md">
          <ArrowUp className="h-4 w-4 mr-2 text-blue-600" />
          <p className="text-gray-800">
            Enter <span className="font-bold">{formatPrice(minNextBid)}</span> or more
          </p>
        </div>
      )}
    </div>
  );
};

export default CurrentBid;
