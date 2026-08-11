import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { Item } from '@/types/item';
import { toast } from 'sonner';
import { Loader2, Home, ChevronRight, ShoppingCart } from 'lucide-react';
import ItemImages from '@/components/items/ItemImages';
import ItemInfo from '@/components/items/ItemInfo';
import BiddingInterface from '@/components/bidding/BiddingInterface';
import BidHistory from '@/components/bidding/BidHistory';
import ItemLocationMap from '@/components/maps/ItemLocationMap';
import NotFound from '@/pages/NotFound';
import { useWebSocketContext } from '@/contexts/WebSocketContext';
import {
  AuctionEvent,
  BidPlacedPayload,
  AuctionEndedPayload,
  CountdownUpdatePayload,
  WebSocketMessage,
} from '@/types/websocket';
import AuctionStatusBanner from '@/components/auction/AuctionStatusBanner';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useCart } from '@/contexts/CartContext';
import { useAuth } from '@/contexts/AuthContext';
import { authFetch } from '@/utils/auth-fetch';
import { globalEvents, EVENTS } from '@/utils/events';

const formatPrice = (price: number) => {
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
};

const ItemDetail: React.FC = () => {
  const { itemId } = useParams<{ itemId: string }>();
  const [item, setItem] = useState<Item | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [timeLeft, setTimeLeft] = useState<number | undefined>();
  const { lastMessage } = useWebSocketContext();
  const { addToCart, isInCart } = useCart();
  const { user, isAuthenticated } = useAuth();

  useEffect(() => {
    if (lastMessage && lastMessage.data?.item_id === itemId) {
      const { type, data } = lastMessage as WebSocketMessage;

      switch (type) {
        case AuctionEvent.BidPlaced: {
          const payload = data as BidPlacedPayload;
          // Show notification for seller and admin, but not for the bidder themselves
          const isOwnBid = payload.bidder_username === user?.username;
          const isSeller = user?.id === item?.seller_user_id;
          const isAdmin = user?.role === 'admin';

          if (!isOwnBid && (isSeller || isAdmin)) {
            toast.info(
              `${payload.bidder_username} placed a new bid of $${Number(payload.amount).toFixed(2)}!`
            );
          }
          setItem((prevItem) => {
            if (!prevItem) return null;
            return {
              ...prevItem,
              currently: payload.current_price,
              number_of_bids: payload.bid_count,
            };
          });
          break;
        }
        case AuctionEvent.AuctionEnded: {
          const payload = data as AuctionEndedPayload;
          toast.success('The auction has ended!');
          setItem((prevItem) => {
            if (!prevItem) return null;
            return {
              ...prevItem,
              status: 'ended',
              winner: payload.winner_username
                ? { id: 0, username: payload.winner_username }
                : undefined,
              currently: payload.winning_bid || prevItem.currently,
            };
          });
          break;
        }
        case AuctionEvent.CountdownUpdate: {
          const payload = data as CountdownUpdatePayload;
          setTimeLeft(payload.time_remaining_seconds);
          break;
        }
        case AuctionEvent.ItemSold: {
          // const payload = data as ItemSoldPayload;
          setItem((prevItem) => {
            if (!prevItem) return null;
            return {
              ...prevItem,
              status: 'sold',
            };
          });
          break;
        }
        default:
          break;
      }
    }
  }, [lastMessage, itemId, item?.listing_type]);

  useEffect(() => {
    const fetchItem = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const response = await fetch(`/api/items/${itemId}`);
        if (!response.ok) {
          if (response.status === 404) {
            setError('404');
          } else {
            throw new Error('Failed to fetch item details');
          }
        } else {
          const data = await response.json();
          // Transform images from nested objects to URL strings (same as Landing page)
          const transformedItem = {
            ...data,
            images: data.images?.map((img: any) => img.url) || [],
          };
          setItem(transformedItem);
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'An unknown error occurred';
        setError(errorMessage);
        toast.error(`Error: ${errorMessage}`);
      } finally {
        setIsLoading(false);
      }
    };
    if (itemId) {
      fetchItem();
    }
  }, [itemId]);

  const handleAddToCart = async () => {
    if (!isAuthenticated) {
      toast.error('Please log in to add items to your cart');
      return;
    }
    if (itemId) {
      await addToCart(itemId);
    }
  };

  const handlePurchase = async () => {
    if (!isAuthenticated) {
      toast.error('Please log in to purchase an item');
      return;
    }
    if (!itemId) return;

    try {
      const response = await authFetch(`/api/items/${itemId}/purchase`, {
        method: 'POST',
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to purchase item');
      }

      const purchasePrice = item?.buy_price || item?.currently || item?.price;
      toast.success(`Purchase completed successfully!
1 item purchased for ${new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(purchasePrice || 0)}
Thank you for your order!`);
      // The WebSocket will handle updating the item status
      // Emit event to refresh profile data
      globalEvents.emit(EVENTS.ITEM_PURCHASED);
    } catch (error) {
      console.error('Purchase failed:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to purchase item');
    }
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-screen">
        <Loader2 className="h-16 w-16 animate-spin text-blue-600" />
      </div>
    );
  }

  if (error === '404' || !item) {
    return <NotFound />;
  }

  if (error) {
    return <div className="text-center text-red-500 py-10">Error loading item: {error}</div>;
  }

  return (
    <div className="container mx-auto p-4 md:p-6 space-y-6">
      <nav className="flex items-center text-sm text-gray-500">
        <Link to="/" className="hover:text-ebay-blue flex items-center">
          <Home className="h-4 w-4 mr-1" /> Home
        </Link>
        <ChevronRight className="h-4 w-4 mx-1" />
        <Link to="/marketplace" className="hover:text-ebay-blue">
          Marketplace
        </Link>
        <ChevronRight className="h-4 w-4 mx-1" />
        <span className="font-semibold text-gray-700 truncate">{item.name}</span>
      </nav>

      {item.listing_type === 'auction' && <AuctionStatusBanner item={item} />}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2 space-y-8">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <ItemImages images={item.images} name={item.name} />
            <ItemInfo item={item} />
          </div>
          <div className="bg-white p-6 rounded-lg shadow-md">
            <h2 className="text-xl font-bold text-gray-900 mb-4 border-b pb-2">Description</h2>
            <p className="text-gray-700 whitespace-pre-wrap">{item.description}</p>
          </div>

          {item.latitude && item.longitude && (
            <ItemLocationMap
              latitude={item.latitude}
              longitude={item.longitude}
              itemName={item.name}
              location={item.location}
            />
          )}
        </div>

        <div className="lg:col-span-1 space-y-6">
          {item.listing_type === 'auction' ? (
            <>
              <BiddingInterface item={item} />
              <BidHistory itemId={item.item_id} />
            </>
          ) : (
            <Card className="shadow-lg">
              <CardHeader>
                <CardTitle>Buy This Item</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <p className="text-sm text-gray-600">Price</p>
                  <p className="text-3xl font-bold text-gray-900">{formatPrice(item.price)}</p>
                </div>
                <div className="space-y-2">
                  {item.status === 'sold' || item.status === 'ended' ? (
                    <Button disabled className="w-full bg-gray-400 cursor-not-allowed">
                      Sold
                    </Button>
                  ) : (
                    <Button
                      className="w-full bg-green-600 hover:bg-green-700"
                      onClick={handlePurchase}
                    >
                      <ShoppingCart className="h-4 w-4 mr-2" />
                      Buy It Now
                    </Button>
                  )}
                  <Button
                    variant="outline"
                    className="w-full"
                    onClick={handleAddToCart}
                    disabled={
                      !isAuthenticated ||
                      (itemId && isInCart(itemId)) ||
                      item.status === 'sold' ||
                      item.status === 'ended'
                    }
                  >
                    {item.status === 'sold' || item.status === 'ended'
                      ? 'Sold'
                      : itemId && isInCart(itemId)
                        ? 'Already in Cart'
                        : 'Add to Cart'}
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
};

export default ItemDetail;
