import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Loader2, CheckCircle } from 'lucide-react';
import { useAuth } from '@/contexts/AuthContext';
import { Bid } from '@/types/bid';
import { toast } from 'sonner';
import { useWebSocketContext } from '@/contexts/WebSocketContext';
import { AuctionEvent, BidPlacedPayload } from '@/types/websocket';

interface BidHistoryProps {
  itemId: string;
}

const INITIAL_VISIBLE_BIDS = 3;

// Helper function to mask username for privacy
const maskUsername = (username: string): string => {
  if (username.length <= 4) return `${username.substring(0, 1)}***`;
  return `${username.substring(0, 2)}***${username.substring(username.length - 2)}`;
};

// Helper to format timestamps into a relative format
const formatRelativeTime = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const seconds = Math.round((now.getTime() - date.getTime()) / 1000);
  const minutes = Math.round(seconds / 60);
  const hours = Math.round(minutes / 60);
  const days = Math.round(hours / 24);

  if (seconds < 60) return `${seconds} seconds ago`;
  if (minutes < 60) return `${minutes} minutes ago`;
  if (hours < 24) return `${hours} hours ago`;
  return `${days} days ago`;
};

const BidHistory: React.FC<BidHistoryProps> = ({ itemId }) => {
  const { user: currentUser } = useAuth();
  const [bids, setBids] = useState<Bid[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [visibleCount, setVisibleCount] = useState(INITIAL_VISIBLE_BIDS);
  const { lastMessage } = useWebSocketContext();

  useEffect(() => {
    // This effect handles incoming WebSocket events to prepend to the bid history
    if (
      lastMessage &&
      lastMessage.data?.item_id === itemId &&
      lastMessage.type === AuctionEvent.BidPlaced
    ) {
      const payload = lastMessage.data as BidPlacedPayload;

      const newBid: Bid = {
        id: new Date().getTime(), // Use timestamp for a temporary unique key
        amount: payload.amount,
        created_at: payload.timestamp,
        user: {
          id: 0, // Note: user ID is not available in the current payload
          username: payload.bidder_username,
        },
      };
      setBids((prevBids) => [newBid, ...prevBids]);
    }
  }, [lastMessage, itemId]);

  useEffect(() => {
    const fetchBids = async () => {
      try {
        const response = await fetch(`/api/items/${itemId}/bids`);
        if (!response.ok) {
          throw new Error('Failed to fetch bid history.');
        }
        const backendData = await response.json();

        // Transform backend data to match frontend Bid interface
        const transformedBids: Bid[] = await Promise.all(
          backendData.map(async (backendBid: any, index: number) => {
            // Fetch username for each bid
            let username = 'Unknown User';
            try {
              const userResponse = await fetch(`/api/users/${backendBid.bidder_user_id}`);
              if (userResponse.ok) {
                const userData = await userResponse.json();
                username = userData.username || 'Unknown User';
              }
            } catch {
              // If user fetch fails, keep default username
            }

            return {
              id: index + 1, // Sequential ID for frontend
              amount: parseFloat(backendBid.amount) || 0,
              created_at: backendBid.time || new Date().toISOString(),
              user: {
                id: index + 1, // Simple sequential ID
                username: username,
              },
            };
          })
        );

        // Sort bids by creation date, newest first
        transformedBids.sort(
          (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
        );
        setBids(transformedBids);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Could not load bids.';
        setError(errorMessage);
        toast.error(errorMessage);
      } finally {
        setIsLoading(false);
      }
    };

    fetchBids();
  }, [itemId]);

  const handleLoadMore = () => {
    setVisibleCount((prevCount) => prevCount + INITIAL_VISIBLE_BIDS);
  };

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Bid History</CardTitle>
        </CardHeader>
        <CardContent className="flex justify-center items-center h-40">
          <Loader2 className="h-8 w-8 animate-spin text-blue-600" />
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Bid History</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-center text-red-500">{error}</p>
        </CardContent>
      </Card>
    );
  }

  const visibleBids = bids.slice(0, visibleCount);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Bid History ({bids.length} bids)</CardTitle>
      </CardHeader>
      <CardContent>
        {bids.length > 0 ? (
          <div className="space-y-2">
            <ul className="space-y-1">
              {visibleBids.map((bid, index) => {
                const isCurrentUserBid = bid.user.id === currentUser?.id;
                const isWinningBid = index === 0;
                const rowClass = isCurrentUserBid
                  ? 'bg-blue-50 border-l-4 border-blue-500'
                  : index % 2 === 0
                    ? 'bg-white'
                    : 'bg-gray-50';

                return (
                  <li
                    key={bid.id}
                    className={`flex items-center justify-between p-3 rounded-md ${rowClass}`}
                  >
                    <div>
                      <p className="font-semibold text-gray-800">
                        ${Number(bid.amount).toFixed(2)}
                      </p>
                      <p className="text-xs text-gray-500">
                        by {maskUsername(bid.user.username)}
                        {isCurrentUserBid && (
                          <span className="font-bold text-blue-600"> (Your Bid)</span>
                        )}
                      </p>
                    </div>
                    <div className="text-right">
                      {isWinningBid && (
                        <p className="text-xs font-bold text-green-600 flex items-center justify-end">
                          <CheckCircle className="h-3 w-3 mr-1" /> Winning
                        </p>
                      )}
                      <p className="text-xs text-gray-400">{formatRelativeTime(bid.created_at)}</p>
                    </div>
                  </li>
                );
              })}
            </ul>
            {bids.length > visibleCount && (
              <Button variant="outline" className="w-full mt-4" onClick={handleLoadMore}>
                Load More Bids
              </Button>
            )}
          </div>
        ) : (
          <p className="text-sm text-center text-gray-500 py-10">No bids have been placed yet.</p>
        )}
      </CardContent>
    </Card>
  );
};

export default BidHistory;
