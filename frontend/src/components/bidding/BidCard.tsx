import React from 'react';
import { Card, CardContent, CardFooter, CardHeader } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useNavigate } from 'react-router-dom';
import { UserBid } from '@/types/userBid';
import { useCountdown } from '@/hooks/useCountdown';
import { Clock, Package } from 'lucide-react';

type BidStatus = 'winning' | 'outbid' | 'won' | 'lost';

interface BidCardProps {
  bid: UserBid;
  status: BidStatus;
}

const formatPrice = (price: number) => {
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
};

const StatusBadge: React.FC<{ status: BidStatus }> = ({ status }) => {
  const styles = {
    winning: 'bg-green-100 text-green-800',
    outbid: 'bg-yellow-100 text-yellow-800',
    won: 'bg-blue-100 text-blue-800',
    lost: 'bg-red-100 text-red-800',
  };
  const text = {
    winning: 'Winning',
    outbid: 'Outbid',
    won: 'You Won!',
    lost: 'Auction Lost',
  };
  return (
    <span className={`px-2 py-1 text-xs font-medium rounded-full ${styles[status]}`}>
      {text[status]}
    </span>
  );
};

const Countdown: React.FC<{ endTime: string }> = ({ endTime }) => {
  const { days, hours, minutes, seconds, isFinished } = useCountdown(endTime);
  if (isFinished) return <span className="font-semibold text-gray-700">Auction Ended</span>;
  return (
    <span className="font-semibold text-gray-700">
      {days}d {hours}h {minutes}m {seconds}s
    </span>
  );
};

const BidCard: React.FC<BidCardProps> = ({ bid, status }) => {
  const navigate = useNavigate();
  const isAuctionActive = bid.item.status === 'active';

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <div className="relative h-40 bg-gray-100 rounded-md">
          {bid.item.images?.[0] ? (
            <img
              src={bid.item.images[0]}
              alt={bid.item.title}
              className="w-full h-full object-cover rounded-md"
            />
          ) : (
            <div className="flex items-center justify-center h-full">
              <Package className="h-16 w-16 text-gray-400" />
            </div>
          )}
        </div>
        <h3
          className="font-semibold text-lg truncate mt-4 cursor-pointer hover:text-blue-600"
          onClick={() => navigate(`/item/${bid.item.id}`)}
          title={bid.item.title}
        >
          {bid.item.title}
        </h3>
      </CardHeader>
      <CardContent className="flex-grow space-y-3">
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-500">Your Bid</span>
          <span className="font-bold text-lg text-blue-600">{formatPrice(bid.amount)}</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-500">Current Bid</span>
          <span className="font-bold text-lg">{formatPrice(bid.item.current_price)}</span>
        </div>
        <div className="flex justify-between items-center text-sm text-gray-500 pt-2 border-t">
          <div className="flex items-center">
            <Clock className="h-4 w-4 mr-1.5" />
            {isAuctionActive && bid.item.end_time ? (
              <Countdown endTime={bid.item.end_time} />
            ) : (
              <span className="font-semibold text-gray-700">Auction Ended</span>
            )}
          </div>
          <StatusBadge status={status} />
        </div>
      </CardContent>
      <CardFooter>
        {status === 'outbid' ? (
          <Button className="w-full" onClick={() => navigate(`/item/${bid.item.id}`)}>
            Bid Again
          </Button>
        ) : (
          <Button
            variant="outline"
            className="w-full"
            onClick={() => navigate(`/item/${bid.item.id}`)}
          >
            View Item
          </Button>
        )}
      </CardFooter>
    </Card>
  );
};

export default BidCard;
