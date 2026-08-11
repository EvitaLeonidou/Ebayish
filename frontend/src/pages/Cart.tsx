import React from 'react';
import { Loader2, ShoppingCart } from 'lucide-react';
// import { CartItem as CartItemType } from '@/types/cart';
import CartItem from '@/components/cart/CartItem';
import CartSummary from '@/components/cart/CartSummary';
import { Button } from '@/components/ui/button';
import { useNavigate } from 'react-router-dom';
import { useCart } from '@/contexts/CartContext';

const Cart: React.FC = () => {
  const navigate = useNavigate();
  const { items, isLoading, removeFromCart } = useCart();

  const handleRemoveItem = async (itemId: string) => {
    await removeFromCart(itemId);
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="container mx-auto p-4 md:p-6">
      <div className="mb-8">
        <h1 className="text-4xl font-bold text-gray-900">Your Shopping Cart</h1>
        <p className="mt-2 text-lg text-gray-600">Review your items and proceed to checkout.</p>
      </div>

      {items.length === 0 ? (
        <div className="text-center py-16 border-2 border-dashed rounded-lg">
          <ShoppingCart className="mx-auto h-16 w-16 text-gray-400" />
          <h3 className="mt-4 text-xl font-semibold text-gray-800">Your cart is empty</h3>
          <p className="mt-1 text-gray-500">Looks like you haven't added anything yet.</p>
          <Button onClick={() => navigate('/marketplace')} className="mt-6">
            Continue Shopping
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-2 bg-white p-6 rounded-lg shadow-md divide-y">
            {items.map((item) => (
              <CartItem key={item.item_id} item={item} onRemove={handleRemoveItem} />
            ))}
          </div>
          <div className="lg:col-span-1">
            <CartSummary items={items} />
          </div>
        </div>
      )}
    </div>
  );
};

export default Cart;
