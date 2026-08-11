import React, { useState, useEffect } from 'react';
import { Item } from '@/types/item';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useAuth } from '@/contexts/AuthContext';
import { toast } from 'sonner';
import { authFetch } from '@/utils/auth-fetch';
import { globalEvents, EVENTS } from '@/utils/events';
import CurrentBid from './CurrentBid';
import BidForm from './BidForm';
import { Lock } from 'lucide-react';

interface BiddingInterfaceProps {
  item: Item;
}

// Helper function to determine the minimum valid bid based on the current price
const calculateMinimumBid = (currentPrice: number, bidCount: number): number => {
  // Ensure we have valid numbers by converting to Number explicitly
  const price = Number(currentPrice) || 0;
  const count = Number(bidCount) || 0;

  console.log('calculateMinimumBid:', { currentPrice, price, bidCount, count });

  if (count === 0) {
    // For the first bid, it must be higher than the starting price (matching backend logic)
    return Math.max(price + 0.01, 0.01);
  }

  // This logic should mirror the backend's bid increment rules
  let increment = 0.5;
  if (price >= 500) increment = 10.0;
  else if (price >= 250) increment = 5.0;
  else if (price >= 100) increment = 2.5;
  else if (price >= 25) increment = 1.0;

  return price + increment;
};

const BiddingInterface: React.FC<BiddingInterfaceProps> = ({ item: initialItem }) => {
  const { user, isAuthenticated } = useAuth();
  const [item, setItem] = useState<Item>(initialItem);

  useEffect(() => {
    setItem(initialItem);
  }, [initialItem]);

  // FIX: Correctly check for ownership using seller_user_id
  const isOwner = user?.id === item.seller_user_id;

  // Check if auction has ended - default to active unless explicitly ended or time has passed
  const auctionEnded = (() => {
    // If status is explicitly set to ended, respect that
    if (item.status === 'ended' || item.status === 'sold') {
      return true;
    }

    // For auction items, check if the end time has passed
    if (item.listing_type === 'auction' && item.ends) {
      const endTime = new Date(item.ends);
      const now = new Date();
      return now > endTime;
    }

    // Default to active (not ended) for all other cases
    return false;
  })();

  const handleBuyNow = async () => {
    if (!isAuthenticated) {
      toast.error('Please log in to purchase an item.');
      return;
    }

    try {
      const response = await authFetch(`/api/items/${item.item_id}/purchase`, {
        method: 'POST',
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to purchase item');
      }

      const purchasePrice = item.buy_price || item.currently || item.price;
      toast.success(`Purchase completed successfully!
1 item purchased for ${new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(purchasePrice)}
Thank you for your order!`);
      // The WebSocket will handle updating the item status
      // Emit event to refresh profile data
      globalEvents.emit(EVENTS.ITEM_PURCHASED);
    } catch (error) {
      console.error('Purchase failed:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to purchase item');
    }
  };

  const handleBidSuccess = (responseData: any) => {
    try {
      // Simple success handling - WebSocket will handle the state update
      console.log('Bid submitted successfully:', responseData);

      // The page will update automatically via WebSocket when the bid is processed
      // No need to reload or manually update state
    } catch (error) {
      console.error('Error in handleBidSuccess:', error);
      toast.error('Bid was placed but there was an error updating the display');
    }
  };

  // Ensure proper number conversion and fix current price logic
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

  const minNextBid = calculateMinimumBid(currentPrice, bidCount);

  console.log('BiddingInterface values:', {
    itemId: item.item_id,
    bidCount,
    currentPrice,
    minNextBid,
    itemPrice: item.price,
    itemCurrently: item.currently,
    listingType: item.listing_type,
  });

  return (
    <Card className="shadow-lg">
      <CardHeader>
        <CardTitle>Auction Details</CardTitle>
        <CardDescription>
          {auctionEnded ? 'This auction has ended.' : 'Place your bid or buy now.'}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <CurrentBid item={item} minNextBid={minNextBid} />

        <div className="border-t pt-4 space-y-4">
          {auctionEnded ? (
            <div className="text-center p-4 bg-gray-100 rounded-md">
              <p className="font-semibold text-gray-800">Auction Ended</p>
              <p className="text-sm text-gray-600">This item is no longer available for bidding.</p>
            </div>
          ) : isOwner ? (
            <div className="text-center p-4 bg-yellow-50 text-yellow-800 rounded-md flex items-center justify-center">
              <Lock className="h-4 w-4 mr-2" />
              <p className="font-semibold">You cannot bid on your own item.</p>
            </div>
          ) : isAuthenticated ? (
            <BidForm
              itemId={item.item_id}
              minBidAmount={minNextBid}
              onBidSuccess={handleBidSuccess}
            />
          ) : (
            <p className="text-center text-sm text-gray-600">
              Please{' '}
              <a href="/login" className="underline font-bold">
                sign in
              </a>{' '}
              to place a bid.
            </p>
          )}

          {item.buy_price && (
            <>
              <div className="relative my-4">
                <div className="absolute inset-0 flex items-center">
                  <span className="w-full border-t" />
                </div>
                <div className="relative flex justify-center text-xs uppercase">
                  <span className="bg-white px-2 text-gray-500">OR</span>
                </div>
              </div>
              <Button
                onClick={handleBuyNow}
                variant="outline"
                className="w-full"
                disabled={auctionEnded || !isAuthenticated}
              >
                {auctionEnded ? 'Sold' : `Buy It Now for ${Number(item.buy_price).toFixed(2)}`}
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export default BiddingInterface;
