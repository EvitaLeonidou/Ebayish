import React, { useState, useEffect } from 'react';
import { Item } from '@/types/item';
import { Tag, User, Star, ShieldCheck, MapPin, Loader2 } from 'lucide-react';

interface ItemInfoProps {
  item: Item;
}

const formatCondition = (condition: string) => {
  if (!condition) return 'Not specified';
  return condition.replace(/_/g, ' ').replace(/\b\w/g, (char) => char.toUpperCase());
};

const StatusBadge: React.FC<{ status?: Item['status'] }> = ({ status }) => {
  const currentStatus = status || 'unknown';
  const statusStyles = {
    active: 'bg-green-100 text-green-800',
    ended: 'bg-red-100 text-red-800',
    sold: 'bg-blue-100 text-blue-800',
    pending: 'bg-yellow-100 text-yellow-800',
    rejected: 'bg-gray-100 text-gray-800',
    unknown: 'bg-gray-100 text-gray-800',
  };
  const currentStyle =
    statusStyles[currentStatus as keyof typeof statusStyles] || 'bg-gray-100 text-gray-800';
  return (
    <span className={`px-2 py-1 text-xs font-medium rounded-full ${currentStyle}`}>
      {currentStatus.charAt(0).toUpperCase() + currentStatus.slice(1)}
    </span>
  );
};

const ItemInfo: React.FC<ItemInfoProps> = ({ item }) => {
  const [sellerUsername, setSellerUsername] = useState<string | null>(null);
  const [isSellerLoading, setIsSellerLoading] = useState(true);

  useEffect(() => {
    const fetchSellerInfo = async () => {
      if (!item.seller_user_id) {
        setSellerUsername('System Listing'); // Handle edge cases where an item might not have a seller
        setIsSellerLoading(false);
        return;
      }

      setIsSellerLoading(true);
      try {
        // This fetch call will now succeed because the backend endpoint exists
        const response = await fetch(`/api/users/${item.seller_user_id}`);
        if (response.ok) {
          const sellerData = await response.json();
          setSellerUsername(sellerData.username); // The backend provides the username directly
        } else {
          // Handle cases where the user ID might be invalid or the API fails
          setSellerUsername('Unknown Seller');
        }
      } catch (error) {
        console.error('Failed to fetch seller info:', error);
        setSellerUsername('Unknown Seller');
      } finally {
        setIsSellerLoading(false);
      }
    };

    fetchSellerInfo();
  }, [item.seller_user_id]); // This effect runs whenever the item's seller ID changes

  // MODIFICATION: Combine the shortened location and country for a clean display.
  const locationParts = [];
  if (item.location) {
    locationParts.push(item.location.split(',')[0]);
  }
  if (item.country) {
    locationParts.push(item.country);
  }
  const displayLocation = locationParts.length > 0 ? locationParts.join(', ') : 'Not Available';

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold text-gray-900">{item.name}</h1>
        <StatusBadge status={item.status} />
      </div>

      <div className="flex items-center space-x-4 text-sm text-gray-600">
        <div className="flex items-center">
          <Tag className="h-4 w-4 mr-1.5 text-gray-500" />
          <span>{item.categories?.[0] || 'Uncategorized'}</span>
        </div>
        <div className="flex items-center">
          <ShieldCheck className="h-4 w-4 mr-1.5 text-gray-500" />
          <span>Condition: {formatCondition(item.condition)}</span>
        </div>
      </div>

      <div className="bg-gray-50 p-4 rounded-lg">
        <div className="flex items-center">
          <User className="h-5 w-5 mr-2 text-gray-500" />
          <div>
            <span className="text-sm text-gray-600">Seller</span>
            {isSellerLoading ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <p className="font-semibold text-blue-600">{sellerUsername}</p>
            )}
          </div>
          {item.seller_rating && (
            <div className="flex items-center ml-auto">
              <Star className="h-4 w-4 text-yellow-400 fill-current" />
              <span className="text-sm font-medium ml-1">
                {Number(item.seller_rating).toFixed(1)}% Positive
              </span>
            </div>
          )}
        </div>
      </div>

      <div className="space-y-2 text-sm">
        <div className="flex items-center text-gray-700">
          <MapPin className="h-4 w-4 mr-2 text-gray-500" />
          <span>Located in: {displayLocation}</span>
        </div>
        {/* You can add more item details here, like shipping info, if needed */}
      </div>
    </div>
  );
};

export default ItemInfo;
