import React, { useState } from 'react';
import { Item } from '@/types/item';
import { useNavigate } from 'react-router-dom';
import { Card, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  Package,
  Clock,
  MapPin,
  Gavel,
  DollarSign,
  ChevronLeft,
  ChevronRight,
  MessageCircle,
  Edit,
} from 'lucide-react';
import { useCountdown } from '@/hooks/useCountdown';
import { useAuth } from '@/contexts/AuthContext';
import { toast } from 'sonner';
import { authFetch } from '@/utils/auth-fetch';
import { globalEvents, EVENTS } from '@/utils/events';

interface ItemCardProps {
  item: Item;
  context?: 'marketplace' | 'purchased' | 'sold' | 'owner';
  contextData?: {
    purchase_date?: string;
    purchase_price?: number;
    seller_username?: string;
    sold_date?: string;
    final_price?: number;
    buyer_username?: string;
    buyer_id?: string;
  };
}

const formatPrice = (price: number) => {
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
};

const Countdown: React.FC<{ endTime: string }> = ({ endTime }) => {
  const { days, hours, minutes, seconds, isFinished } = useCountdown(endTime);

  if (isFinished) {
    return <span className="font-semibold text-red-600">Auction Ended</span>;
  }

  const isEndingSoon = days === 0 && hours < 1;
  const textColor = isEndingSoon ? 'text-red-600' : 'text-gray-700';

  return (
    <span className={`font-semibold ${textColor}`}>
      {days > 0 && `${days}d `}
      {hours > 0 && `${hours}h `}
      {minutes > 0 && `${minutes}m `}
      {seconds}s left
    </span>
  );
};

const ItemCard: React.FC<ItemCardProps> = ({ item, context = 'marketplace', contextData }) => {
  const navigate = useNavigate();
  const { isAuthenticated } = useAuth();
  const [currentImageIndex, setCurrentImageIndex] = useState(0);

  const images = item.images && item.images.length > 0 ? item.images : [];
  const hasMultipleImages = images.length > 1;

  // Use relative path for image URLs (Vite proxy will handle routing to backend)
  const getImageUrl = (imagePath: string | null) => {
    if (!imagePath) return null;
    // Return the path as-is for Vite proxy to handle
    return imagePath;
  };

  const nextImage = (e: React.MouseEvent) => {
    e.stopPropagation();
    setCurrentImageIndex((prev) => (prev + 1) % images.length);
  };

  const prevImage = (e: React.MouseEvent) => {
    e.stopPropagation();
    setCurrentImageIndex((prev) => (prev - 1 + images.length) % images.length);
  };

  // const goToImage = (index: number, e: React.MouseEvent) => {
  //   e.stopPropagation();
  //   setCurrentImageIndex(index);
  // };

  const currentImageUrl = images.length > 0 ? getImageUrl(images[currentImageIndex]) : null;
  const isSold = item.status === 'sold' || item.status === 'ended';

  // MODIFICATION: Format the location to show only the part before the first comma.
  const displayLocation = item.location ? item.location.split(',')[0] : 'Not specified';

  const handlePurchase = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!isAuthenticated) {
      toast.error('Please log in to purchase an item');
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
      // Emit event to refresh profile data
      globalEvents.emit(EVENTS.ITEM_PURCHASED);
    } catch (error) {
      console.error('Purchase failed:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to purchase item');
    }
  };

  const handleBidClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigate(`/item/${item.item_id}`);
  };

  const handleMessage = (e: React.MouseEvent) => {
    e.stopPropagation();

    console.log('handleMessage - context:', context);
    console.log('handleMessage - contextData:', contextData);
    console.log('handleMessage - item.seller_user_id:', item.seller_user_id);

    let targetUserId: string | null = null;

    if (context === 'purchased' && item.seller_user_id) {
      targetUserId = item.seller_user_id;
    } else if (context === 'sold' && contextData?.buyer_id) {
      targetUserId = String(contextData.buyer_id);
    }

    console.log('handleMessage - targetUserId:', targetUserId);

    if (targetUserId) {
      navigate(`/messaging/${targetUserId}`);
    } else {
      toast.error('Unable to start conversation');
    }
  };

  const isListFormat = context === 'purchased' || context === 'sold' || context === 'owner';

  const handleEdit = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigate(`/sell/edit/${item.item_id}`);
  };

  // Check if item can be edited (fixed price items always editable, auctions only if no bids)
  const canEdit =
    context === 'owner' &&
    (item.listing_type === 'fixed_price' ||
      (item.listing_type === 'auction' &&
        (item.number_of_bids === 0 || item.number_of_bids == null)));

  const trackCategoryView = async (itemId: string) => {
    if (!isAuthenticated) return;
    try {
      await authFetch('/api/track-view', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ item_id: itemId }),
      });
    } catch (error) {
      // Silently fail - tracking shouldn't disrupt UX
      console.debug('Failed to track category view:', error);
    }
  };

  const handleCardClick = () => {
    trackCategoryView(item.item_id);
    navigate(`/item/${item.item_id}`);
  };

  return (
    <Card
      className={
        isListFormat
          ? 'flex overflow-hidden transition-all duration-300 hover:shadow-lg cursor-pointer'
          : 'flex flex-col overflow-hidden transition-all duration-300 hover:shadow-xl cursor-pointer'
      }
      onClick={handleCardClick}
    >
      <div
        className={
          isListFormat
            ? 'relative w-24 h-24 bg-gray-100 flex items-center justify-center flex-shrink-0 group'
            : 'relative h-48 bg-gray-100 flex items-center justify-center group'
        }
      >
        {currentImageUrl ? (
          <>
            <img
              src={currentImageUrl}
              alt={item.name}
              className="w-full h-full object-contain"
              onError={(e) => {
                console.warn(`Failed to load image: ${currentImageUrl}`);
                // Hide the img and show fallback
                e.currentTarget.style.display = 'none';
                const fallback = e.currentTarget.nextElementSibling as HTMLElement;
                if (fallback) fallback.style.display = 'flex';
              }}
            />
            <div
              className="absolute inset-0 bg-gray-100 flex items-center justify-center"
              style={{ display: 'none' }}
            >
              <Package className="h-16 w-16 text-gray-400" />
            </div>
          </>
        ) : (
          <Package className="h-16 w-16 text-gray-400" />
        )}

        {/* Navigation arrows - only show if multiple images */}
        {hasMultipleImages && (
          <>
            <button
              onClick={prevImage}
              className="absolute left-2 top-1/2 transform -translate-y-1/2 bg-black bg-opacity-50 hover:bg-opacity-70 text-white p-1 rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-200"
            >
              <ChevronLeft className="h-4 w-4" />
            </button>
            <button
              onClick={nextImage}
              className="absolute right-2 top-1/2 transform -translate-y-1/2 bg-black bg-opacity-50 hover:bg-opacity-70 text-white p-1 rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-200"
            >
              <ChevronRight className="h-4 w-4" />
            </button>
          </>
        )}

        {/* Image counter - only show if multiple images */}
        {hasMultipleImages && (
          <div className="absolute top-2 right-2 bg-black bg-opacity-50 text-white text-xs px-2 py-1 rounded">
            {currentImageIndex + 1}/{images.length}
          </div>
        )}

        {/* Sold overlay */}
        {isSold && (
          <div className="absolute inset-0 bg-black bg-opacity-60 flex items-center justify-center">
            <span className="text-white text-xl font-bold">SOLD</span>
          </div>
        )}
      </div>
      <CardContent
        className={
          isListFormat
            ? 'p-4 flex-grow flex flex-col justify-between'
            : 'p-4 flex-grow flex flex-col'
        }
      >
        <div className={isListFormat ? 'flex-grow' : ''}>
          <h3
            className={
              isListFormat
                ? 'font-semibold text-lg text-gray-900 line-clamp-1'
                : 'font-semibold text-lg text-gray-900 line-clamp-2 h-14'
            }
            title={item.name}
          >
            {item.name}
          </h3>
          {!isListFormat && (
            <div className="flex-grow mt-2 space-y-2 text-sm">
              <div className="flex items-center text-gray-600">
                <MapPin className="h-4 w-4 mr-2 flex-shrink-0" />
                {/* MODIFICATION: Use the formatted displayLocation */}
                <span>{displayLocation}</span>
              </div>
              {/* Conditional rendering for auction details */}
              {item.listing_type === 'auction' && item.ends && (
                <>
                  <div className="flex items-center text-gray-600">
                    <Clock className="h-4 w-4 mr-2 flex-shrink-0" />
                    <Countdown endTime={item.ends} />
                  </div>
                  <div className="flex items-center text-gray-600">
                    <Gavel className="h-4 w-4 mr-2 flex-shrink-0" />
                    <span>{item.number_of_bids ?? 0} bids</span>
                  </div>
                </>
              )}
              {item.buy_price && (
                <div className="flex items-center text-green-700">
                  <DollarSign className="h-4 w-4 mr-2 flex-shrink-0" />
                  <span className="font-medium">Buy now for {formatPrice(item.buy_price)}</span>
                </div>
              )}
            </div>
          )}
        </div>
      </CardContent>
      <CardFooter
        className={
          isListFormat ? 'p-4 bg-gray-50 border-l flex-shrink-0 w-80' : 'p-4 bg-gray-50 border-t'
        }
      >
        <div className="w-full">
          {context === 'purchased' ? (
            <>
              <p className="text-xs text-gray-500">Purchased</p>
              <div className="flex justify-between items-center">
                <div>
                  <p className="text-lg font-bold text-gray-800">
                    {formatPrice(contextData?.purchase_price ?? item.price)}
                  </p>
                  <p className="text-xs text-gray-500">
                    on{' '}
                    {contextData?.purchase_date
                      ? new Date(contextData.purchase_date).toLocaleDateString()
                      : 'Unknown date'}
                  </p>
                </div>
                <Button size="sm" variant="outline" onClick={handleMessage}>
                  <MessageCircle className="h-4 w-4 mr-1" />
                  Message {contextData?.seller_username}
                </Button>
              </div>
            </>
          ) : context === 'sold' ? (
            <>
              <p className="text-xs text-gray-500">Sold</p>
              <div className="flex justify-between items-center">
                <div>
                  <p className="text-lg font-bold text-gray-800">
                    {formatPrice(contextData?.final_price ?? item.price)}
                  </p>
                  <p className="text-xs text-gray-500">
                    on{' '}
                    {contextData?.sold_date
                      ? new Date(contextData.sold_date).toLocaleDateString()
                      : 'Unknown date'}
                  </p>
                </div>
                <Button size="sm" variant="outline" onClick={handleMessage}>
                  <MessageCircle className="h-4 w-4 mr-1" />
                  Message {contextData?.buyer_username}
                </Button>
              </div>
            </>
          ) : context === 'owner' ? (
            // Owner view - show edit button for applicable items
            <>
              <p className="text-xs text-gray-500">
                {item.listing_type === 'auction' ? 'Current Bid' : 'Price'}
              </p>
              <div className="flex justify-between items-center">
                <p className="text-2xl font-bold text-gray-800">
                  {item.listing_type === 'auction'
                    ? formatPrice(item.currently ?? item.price)
                    : formatPrice(item.price)}
                </p>
                <div className="flex gap-2">
                  {canEdit && (
                    <Button size="sm" variant="outline" onClick={handleEdit}>
                      <Edit className="h-4 w-4 mr-1" />
                      Edit
                    </Button>
                  )}
                  <Button size="sm" variant="outline" onClick={handleBidClick}>
                    View Details
                  </Button>
                </div>
              </div>
              {item.listing_type === 'auction' && (
                <p className="text-xs text-gray-500 mt-1">
                  {item.number_of_bids ?? 0} bids
                  {canEdit && <span className="text-green-600 ml-2">• Can edit (no bids)</span>}
                  {!canEdit && item.listing_type === 'auction' && (
                    <span className="text-orange-600 ml-2">• Cannot edit (has bids)</span>
                  )}
                </p>
              )}
            </>
          ) : item.listing_type === 'auction' ? (
            <>
              <p className="text-xs text-gray-500">Current Bid</p>
              <div className="flex justify-between items-center">
                <p className="text-2xl font-bold text-gray-800">
                  {formatPrice(item.currently ?? item.price)}
                </p>
                <Button
                  size="sm"
                  className="bg-ebay-blue hover:bg-blue-700"
                  disabled={isSold}
                  onClick={handleBidClick}
                >
                  {isSold ? 'Sold' : 'Place Bid'}
                </Button>
              </div>
            </>
          ) : (
            <>
              <p className="text-xs text-gray-500">Price</p>
              <div className="flex justify-between items-center">
                <p className="text-2xl font-bold text-gray-800">{formatPrice(item.price)}</p>
                <Button
                  size="sm"
                  className="bg-green-600 hover:bg-green-700"
                  disabled={isSold}
                  onClick={handlePurchase}
                >
                  {isSold ? 'Sold' : 'Buy Now'}
                </Button>
              </div>
            </>
          )}
        </div>
      </CardFooter>
    </Card>
  );
};

export default ItemCard;
