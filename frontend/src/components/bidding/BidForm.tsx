import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { DollarSign, Loader2, X, AlertTriangle } from 'lucide-react';
import { toast } from 'sonner';
import { useAuth } from '@/contexts/AuthContext';

interface BidFormProps {
  itemId: string; // The ID of the item being bid on
  minBidAmount: number; // The minimum valid bid amount
  onBidSuccess: (newBidResponse: any) => void; // Callback after a successful bid
}

const BidForm: React.FC<BidFormProps> = ({ itemId, minBidAmount, onBidSuccess }) => {
  const [bidAmount, setBidAmount] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [showConfirmation, setShowConfirmation] = useState(false);
  const { user, token } = useAuth(); // Get the current authenticated user from the context

  const handleBidSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    // --- 1. Input Validation ---
    const amount = parseFloat(bidAmount);
    console.log('Bid validation:', {
      amount,
      minBidAmount,
      isNaN: isNaN(amount),
      lessThan: amount < minBidAmount,
    });

    if (isNaN(amount) || amount <= 0) {
      const errorMessage = 'Please enter a valid bid amount.';
      setError(errorMessage);
      toast.error(errorMessage);
      return;
    }

    if (amount < minBidAmount) {
      const errorMessage = `Your bid must be at least $${Number(minBidAmount).toFixed(2)}.`;
      setError(errorMessage);
      toast.error(errorMessage);
      return;
    }

    if (!user) {
      toast.error('You must be logged in to place a bid.');
      return;
    }

    // Show confirmation dialog instead of submitting immediately
    setShowConfirmation(true);
  };

  const handleConfirmBid = async () => {
    setIsLoading(true);

    const amount = parseFloat(bidAmount);

    try {
      // --- Fetch latest item data to ensure we have current price ---
      console.log('Fetching latest item data before bid...');

      let latestCurrentPrice = null;
      try {
        const itemResponse = await fetch(`/api/items/${itemId}`);
        console.log('Item fetch response status:', itemResponse.status);

        if (itemResponse.ok) {
          const latestItem = await itemResponse.json();
          console.log('Latest item data:', {
            currently: latestItem.currently,
            price: latestItem.price,
            number_of_bids: latestItem.number_of_bids,
            listing_type: latestItem.listing_type,
          });

          // Check if our bid is still valid against the latest data
          // Use the same logic as frontend components to match backend validation
          latestCurrentPrice = (() => {
            if (latestItem.listing_type === 'auction') {
              const priceValue = Number(latestItem.price || 0);
              const currentlyValue = Number(latestItem.currently || 0);
              return currentlyValue > 0 ? currentlyValue : priceValue;
            }
            return Number(latestItem.price || 0);
          })();

          console.log('Current price validation:', {
            userBidAmount: amount,
            latestCurrentPrice,
            bidCount: latestItem.number_of_bids,
            itemCurrently: latestItem.currently,
            itemPrice: latestItem.price,
            isValidBid: amount > latestCurrentPrice,
          });

          if (amount <= latestCurrentPrice) {
            const errorMessage = `Your bid must be higher than the current price of $${Number(latestCurrentPrice).toFixed(2)}. You bid $${amount}.`;
            toast.error(errorMessage);
            setError(errorMessage);
            setIsLoading(false);
            setShowConfirmation(false);
            return;
          }
        } else {
          console.error('Failed to fetch latest item data:', itemResponse.status);
          // Continue with bid submission even if we can't fetch latest data
        }
      } catch (fetchError) {
        console.error('Error fetching latest item data:', fetchError);
        // Continue with bid submission even if fetch fails
      }
      // --- 2. Construct API Payload ---
      // Backend expects bidder_user_id as UUID (without quotes in JSON)
      // The user.id should already be a valid UUID string

      const payload = {
        bidder_user_id: user.id, // UUID string - backend will parse as UUID
        bidder_rating: null, // Optional field required by backend
        time: new Date().toISOString(), // Send current time in ISO format
        amount: Number(amount), // Ensure it's a number
        bidder_location: null, // Optional field required by backend
        bidder_country: null, // Optional field required by backend
      };

      // --- 3. Send API Request ---
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }

      console.log('=== BID SUBMISSION DEBUG ===');
      console.log('Sending bid payload:', payload);
      console.log('User ID type:', typeof user.id, 'Value:', user.id);
      console.log('Bid amount entered:', bidAmount);
      console.log('Bid amount parsed:', amount);
      console.log('Min bid amount required:', minBidAmount);
      console.log('Item ID:', itemId);
      console.log('Latest current price from fetch:', latestCurrentPrice);
      console.log('=== END DEBUG ===');

      const response = await fetch(`/api/items/${itemId}/bids`, {
        method: 'POST',
        headers,
        body: JSON.stringify(payload),
      });

      console.log('Bid response status:', response.status);

      // Clone the response so we can read the body multiple times if needed
      // const responseClone = response.clone();

      let responseData;
      let responseText;

      try {
        // First try to read as text since backend might return plain text errors
        responseText = await response.text();
        console.log('Bid response text:', responseText);

        // Then try to parse as JSON if it's valid JSON
        if (responseText.trim().startsWith('{') || responseText.trim().startsWith('[')) {
          responseData = JSON.parse(responseText);
          console.log('Bid response data:', responseData);
        }
      } catch (e) {
        console.error('Failed to parse response:', e);
      }

      if (!response.ok) {
        // Use the error message from the backend if available
        const errorMessage = responseData?.message || responseText || 'Bid submission failed.';
        console.error('Bid failed with error:', errorMessage);
        throw new Error(errorMessage);
      }

      // --- 4. Handle Success ---
      toast.success('Your bid was placed successfully!');
      onBidSuccess(responseData); // Pass the full API response to the parent component
      setBidAmount(''); // Clear the input field
      setShowConfirmation(false); // Close the confirmation dialog
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'An unknown error occurred.';
      toast.error(errorMessage);
      setError(errorMessage);
      setShowConfirmation(false); // Close the confirmation dialog on error
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancelBid = () => {
    setShowConfirmation(false);
    setIsLoading(false); // Reset loading state when canceling
  };

  return (
    <>
      {showConfirmation && (
        <div
          className="fixed inset-0 bg-black/60 flex justify-center items-center z-50 animate-in fade-in"
          onClick={handleCancelBid}
        >
          <Card
            className="w-full max-w-md m-4 animate-in zoom-in-95"
            onClick={(e) => e.stopPropagation()}
          >
            <CardHeader className="flex flex-row items-start justify-between">
              <div className="flex items-center space-x-2">
                <AlertTriangle className="h-5 w-5 text-amber-600" />
                <CardTitle className="text-lg">Confirm Your Bid</CardTitle>
              </div>
              <Button variant="outline" size="sm" onClick={handleCancelBid}>
                <X className="h-4 w-4" />
              </Button>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-gray-600">
                Are you sure you want to place a bid of{' '}
                <span className="font-semibold text-gray-900">
                  ${(parseFloat(bidAmount) || 0).toFixed(2)}
                </span>
                ?
              </p>
              <p className="text-sm text-gray-500">
                This action cannot be undone. By confirming, you agree to purchase this item if you
                win the auction.
              </p>
              <div className="flex space-x-3 pt-2">
                <Button variant="outline" onClick={handleCancelBid} className="flex-1">
                  Cancel
                </Button>
                <Button
                  onClick={handleConfirmBid}
                  className="flex-1 bg-blue-600 hover:bg-blue-700"
                  disabled={isLoading}
                >
                  {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  {isLoading ? 'Placing...' : 'Confirm Bid'}
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      <form onSubmit={handleBidSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="bid-amount">Your Maximum Bid</Label>
          <div className="relative">
            <DollarSign className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
            <Input
              id="bid-amount"
              type="number"
              step="0.01" // Allow decimal bids
              min={minBidAmount}
              placeholder={`$${Number(minBidAmount).toFixed(2)} or more`}
              className={`pl-9 ${error ? 'border-red-500 focus:border-red-500' : ''} [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none [-moz-appearance:textfield]`}
              value={bidAmount}
              onChange={(e) => setBidAmount(e.target.value)}
              disabled={isLoading}
              required
            />
          </div>
          {error && <p className="text-xs text-red-600 mt-1">{error}</p>}
        </div>
        <Button type="submit" className="w-full bg-blue-600 hover:bg-blue-700" disabled={isLoading}>
          {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {isLoading ? 'Placing Bid...' : 'Place Bid'}
        </Button>
      </form>
    </>
  );
};

export default BidForm;
