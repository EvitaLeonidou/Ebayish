import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { cartService, CartItem } from '@/services/cartService';
import { toast } from 'sonner';
import { useAuth } from './AuthContext';

interface CartContextType {
  items: CartItem[];
  isLoading: boolean;
  itemCount: number;
  addToCart: (itemId: string) => Promise<void>;
  removeFromCart: (itemId: string) => Promise<void>;
  clearCart: () => Promise<void>;
  refreshCart: () => Promise<void>;
  isInCart: (itemId: string) => boolean;
}

const CartContext = createContext<CartContextType | undefined>(undefined);

interface CartProviderProps {
  children: ReactNode;
}

export function CartProvider({ children }: CartProviderProps) {
  const [items, setItems] = useState<CartItem[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const { isAuthenticated } = useAuth();

  const itemCount = items.length;

  const refreshCart = async () => {
    if (!isAuthenticated) {
      setItems([]);
      return;
    }

    setIsLoading(true);
    try {
      const cartItems = await cartService.getCart();
      setItems(cartItems);
    } catch (error) {
      console.error('Failed to refresh cart:', error);
      toast.error('Failed to load cart');
      setItems([]);
    } finally {
      setIsLoading(false);
    }
  };

  const addToCart = async (itemId: string) => {
    try {
      await cartService.addToCart(itemId);
      await refreshCart();
      toast.success('Item added to cart');
    } catch (error: any) {
      console.error('Failed to add to cart:', error);
      if (error.response?.status === 409) {
        toast.error('Item is already in your cart');
      } else if (error.response?.status === 403) {
        toast.error('You cannot add your own item to the cart');
      } else if (error.response?.status === 404) {
        toast.error('Item not found');
      } else if (error.response?.status === 401) {
        toast.error('Please log in to add items to cart');
      } else {
        toast.error('Failed to add item to cart');
      }
    }
  };

  const removeFromCart = async (itemId: string) => {
    try {
      await cartService.removeFromCart(itemId);
      await refreshCart();
      toast.success('Item removed from cart');
    } catch (error: any) {
      console.error('Failed to remove from cart:', error);
      if (error.response?.status === 404) {
        toast.error('Item not found in cart');
      } else if (error.response?.status === 401) {
        toast.error('Please log in to manage your cart');
      } else {
        toast.error('Failed to remove item from cart');
      }
    }
  };

  const clearCart = async () => {
    try {
      await cartService.clearCart();
      setItems([]);
      toast.success('Cart cleared');
    } catch (error: any) {
      console.error('Failed to clear cart:', error);
      if (error.response?.status === 401) {
        toast.error('Please log in to manage your cart');
      } else {
        toast.error('Failed to clear cart');
      }
    }
  };

  const isInCart = (itemId: string): boolean => {
    return items.some((item) => item.item_id === itemId);
  };

  useEffect(() => {
    if (isAuthenticated) {
      refreshCart();
    } else {
      setItems([]);
    }
  }, [isAuthenticated]);

  const value = {
    items,
    isLoading,
    itemCount,
    addToCart,
    removeFromCart,
    clearCart,
    refreshCart,
    isInCart,
  };

  return <CartContext.Provider value={value}>{children}</CartContext.Provider>;
}

export const useCart = () => {
  const context = useContext(CartContext);
  if (context === undefined) {
    throw new Error('useCart must be used within a CartProvider');
  }
  return context;
};
