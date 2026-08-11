import React, { useState, useEffect, useMemo } from 'react';
import { toast } from 'sonner';
import { Loader2, SearchX } from 'lucide-react';
import { UserBid } from '@/types/userBid';
import BidCard from '@/components/bidding/BidCard';
import { Button } from '@/components/ui/button';

type BidStatusFilter = 'all' | 'active' | 'winning' | 'outbid' | 'won' | 'lost';

const MyBids: React.FC = () => {
  const [bids, setBids] = useState<UserBid[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [filter, setFilter] = useState<BidStatusFilter>('all');

  useEffect(() => {
    const fetchMyBids = async () => {
      try {
        const response = await fetch('/api/bids');
        if (!response.ok) {
          throw new Error('Failed to fetch your bids.');
        }
        const data: UserBid[] = await response.json();
        data.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
        setBids(data);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred.';
        toast.error(errorMessage);
      } finally {
        setIsLoading(false);
      }
    };

    fetchMyBids();
  }, []);

  const getBidStatus = (bid: UserBid): 'winning' | 'outbid' | 'won' | 'lost' => {
    const isAuctionActive = bid.item.status === 'active';
    const isWinning = bid.amount >= bid.item.current_price;

    if (isAuctionActive) {
      return isWinning ? 'winning' : 'outbid';
    } else {
      return isWinning ? 'won' : 'lost';
    }
  };

  const filteredBids = useMemo(() => {
    if (filter === 'all') return bids;
    if (filter === 'active') {
      return bids.filter((bid) => bid.item.status === 'active');
    }
    return bids.filter((bid) => getBidStatus(bid) === filter);
  }, [bids, filter]);

  const FilterButton: React.FC<{ status: BidStatusFilter; label: string }> = ({
    status,
    label,
  }) => (
    <Button variant={filter === status ? 'default' : 'outline'} onClick={() => setFilter(status)}>
      {label}
    </Button>
  );

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 md:p-6 space-y-6">
      <div className="text-center">
        <h1 className="text-4xl font-bold text-gray-900">My Bids</h1>
        <p className="mt-2 text-lg text-gray-600">Track and manage your auction activity.</p>
      </div>

      <div className="flex flex-wrap justify-center gap-2">
        <FilterButton status="all" label="All Bids" />
        <FilterButton status="active" label="Active" />
        <FilterButton status="winning" label="Winning" />
        <FilterButton status="outbid" label="Outbid" />
        <FilterButton status="won" label="Won" />
        <FilterButton status="lost" label="Lost" />
      </div>

      {filteredBids.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredBids.map((bid) => (
            <BidCard key={bid.id} bid={bid} status={getBidStatus(bid)} />
          ))}
        </div>
      ) : (
        <div className="text-center py-16 border-2 border-dashed rounded-lg">
          <SearchX className="mx-auto h-16 w-16 text-gray-400" />
          <h3 className="mt-4 text-xl font-semibold text-gray-800">No Bids Found</h3>
          <p className="mt-1 text-gray-500">
            {filter === 'all'
              ? "You haven't placed any bids yet."
              : 'No bids match the selected filter.'}
          </p>
        </div>
      )}
    </div>
  );
};

export default MyBids;
