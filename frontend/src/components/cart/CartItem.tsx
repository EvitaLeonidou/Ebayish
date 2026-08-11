import React from 'react';
import { CartItem as CartItemType } from '@/types/cart';
import { Button } from '@/components/ui/button';
import { Trash2, Package } from 'lucide-react';
import { Link } from 'react-router-dom';

interface CartItemProps {
  item: CartItemType;
  onRemove: (itemId: string) => void;
}

const formatPrice = (price: number) => {
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
};

const CartItem: React.FC<CartItemProps> = ({ item, onRemove }) => {
  const displayPrice = Number(item.buy_price || item.currently);
  const primaryImage = item.images && item.images.length > 0 ? item.images[0] : null;

  // Use relative path for image URLs (Vite proxy will handle routing to backend)
  const getImageUrl = (imagePath: string | null) => {
    if (!imagePath) return null;
    // Return the path as-is for Vite proxy to handle
    return imagePath;
  };

  const imageUrl = getImageUrl(primaryImage);

  return (
    <div className="flex items-start gap-4 py-4">
      <div className="w-24 h-24 bg-gray-100 rounded-md flex-shrink-0 overflow-hidden">
        {imageUrl ? (
          <>
            <img
              src={imageUrl}
              alt={item.name}
              className="w-full h-full object-cover"
              onError={(e) => {
                console.warn(`Failed to load image: ${imageUrl}`);
                // Hide the img and show fallback
                e.currentTarget.style.display = 'none';
                const fallback = e.currentTarget.nextElementSibling as HTMLElement;
                if (fallback) fallback.style.display = 'flex';
              }}
            />
            <div
              className="w-full h-full bg-gray-100 flex items-center justify-center"
              style={{ display: 'none' }}
            >
              <Package className="h-12 w-12 text-gray-400" />
            </div>
          </>
        ) : (
          <div className="flex items-center justify-center h-full">
            <Package className="h-12 w-12 text-gray-400" />
          </div>
        )}
      </div>
      <div className="flex-grow">
        <Link to={`/item/${item.item_id}`} className="hover:text-blue-600">
          <h3 className="font-semibold text-lg text-gray-900 line-clamp-2">{item.name}</h3>
        </Link>
        <div className="mt-2">
          {item.listing_type === 'fixed_price' ? (
            <p className="text-sm text-gray-600">
              Price: {formatPrice(Number(item.buy_price || item.currently))}
            </p>
          ) : (
            <div>
              {item.buy_price && (
                <p className="text-sm text-gray-600">
                  Buy Now Price: {formatPrice(Number(item.buy_price))}
                </p>
              )}
              <p className="text-sm text-gray-500">
                Current Bid: {formatPrice(Number(item.currently))}
              </p>
            </div>
          )}
        </div>
      </div>
      <div className="text-right">
        <p className="font-bold text-lg text-gray-900">{formatPrice(displayPrice)}</p>
        <Button
          variant="outline"
          size="sm"
          className="mt-2 text-red-600 hover:text-red-700"
          onClick={() => onRemove(item.item_id)}
        >
          <Trash2 className="h-4 w-4 mr-1" />
          Remove
        </Button>
      </div>
    </div>
  );
};

export default CartItem;
