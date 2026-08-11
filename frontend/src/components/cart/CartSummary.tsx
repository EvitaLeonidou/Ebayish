import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { CartItem } from '@/types/cart';
import { ShoppingCart, Loader2 } from 'lucide-react';
import { Separator } from '@/components/ui/separator';
import { toast } from 'sonner';
import { useCart } from '@/contexts/CartContext';
import { authFetch } from '@/utils/auth-fetch';

interface CartSummaryProps {
  items: CartItem[];
}

const formatPrice = (price: number) => {
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
};

const CartSummary: React.FC<CartSummaryProps> = ({ items }) => {
  const [isProcessing, setIsProcessing] = useState(false);
  const { clearCart } = useCart();

  const subtotal = items.reduce((acc, item) => {
    const itemPrice = item.buy_price || item.currently;
    return acc + Number(itemPrice);
  }, 0);
  const shipping = 5.99; // Mocked shipping cost
  const total = subtotal + shipping;

  const handleCompletePurchase = async () => {
    setIsProcessing(true);
    try {
      // Purchase each item individually using the existing purchase endpoint
      const purchasePromises = items.map((item) =>
        authFetch(`/api/items/${item.item_id}/purchase`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
        })
      );

      const responses = await Promise.all(purchasePromises);

      // Check if all purchases succeeded
      for (const response of responses) {
        if (!response.ok) {
          const errorData = await response.json();
          throw new Error(errorData.message || 'Failed to complete purchase');
        }
      }

      // Clear cart and show success
      await clearCart();
      toast.success(`Purchase completed successfully!
${items.length} item(s) purchased for ${formatPrice(total)}
Thank you for your order!`);
    } catch (error) {
      console.error('Purchase failed:', error);
      const errorMessage =
        error instanceof Error ? error.message : 'An error occurred during purchase';
      toast.error(`Purchase failed: ${errorMessage}`);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <Card className="sticky top-24 shadow-md">
      <CardHeader>
        <CardTitle>Order Summary</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex justify-between text-gray-600">
          <span>Subtotal</span>
          <span>{formatPrice(subtotal)}</span>
        </div>
        <div className="flex justify-between text-gray-600">
          <span>Shipping</span>
          <span>{formatPrice(shipping)}</span>
        </div>
        <Separator />
        <div className="flex justify-between font-bold text-lg text-gray-900">
          <span>Total</span>
          <span>{formatPrice(total)}</span>
        </div>
      </CardContent>
      <CardFooter>
        <Button
          className="w-full bg-blue-600 hover:bg-blue-700"
          disabled={items.length === 0 || isProcessing}
          onClick={handleCompletePurchase}
        >
          {isProcessing ? (
            <>
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              Processing...
            </>
          ) : (
            <>
              <ShoppingCart className="h-4 w-4 mr-2" />
              Complete Purchase
            </>
          )}
        </Button>
      </CardFooter>
    </Card>
  );
};

export default CartSummary;
