import { Button } from '@/components/ui/button';
import Header from '@/components/Header';
import { Loader2 } from 'lucide-react';
import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
// import { Category } from '@/types/category';
import { Item } from '@/types/item';
import { toast } from 'sonner';
import ItemCard from '@/components/items/ItemCard';
import { useAuth } from '@/contexts/AuthContext';
import { authFetch } from '@/utils/auth-fetch';
const Landing: React.FC = () => {
  const navigate = useNavigate();
  // const [categories, setCategories] = useState<Category[]>([]);
  const [featuredItems, setFeaturedItems] = useState<Item[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const { user } = useAuth();
  useEffect(() => {
    const fetchData = async () => {
      try {
        // Use recommendations for authenticated users, latest items for guests
        const endpoint = user ? `/api/recommendations/${user.id}?limit=4` : '/api/items';
        const itemResponse = user ? await authFetch(endpoint) : await fetch(endpoint);
        if (!itemResponse.ok) {
          throw new Error('Failed to fetch landing page data');
        }
        const responseData = await itemResponse.json();

        // Handle different response formats
        const itemData = user ? responseData.recommendations : responseData;

        // Transform items to extract image URLs from the nested image objects
        const itemsWithImages = itemData.map((item: any) => ({
          ...item,
          images: user
            ? item.images || [] // Recommendations already have URL strings
            : item.images?.map((img: any) => img.url) || [], // Regular items have objects with url property
        }));

        // For recommendations, items are already filtered and sorted
        // For latest items, filter out sold/ended items and user's own items
        const sortedItems = user
          ? itemsWithImages // Recommendations are pre-filtered
          : itemsWithImages
              .filter((item: any) => {
                // Filter out sold/ended items
                if (item.status === 'sold' || item.status === 'ended') return false;
                // Filter out user's own listings
                if (user && item.seller_user_id === user.id) return false;
                return true;
              })
              .sort(
                (a: any, b: any) =>
                  new Date(b.created_at || b.started).getTime() -
                  new Date(a.created_at || a.started).getTime()
              )
              .slice(0, 4);

        setFeaturedItems(sortedItems);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Could not load page data.');
      } finally {
        setIsLoading(false);
      }
    };
    fetchData();
  }, [user]);
  return (
    <div className="min-h-screen bg-gray-50 w-full overflow-x-hidden">
      <Header />
      {/* Hero Section */}
      <section className="bg-white text-gray-800 py-16 w-full border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-4xl md:text-6xl font-bold mb-4">
            Buy.
            <span className="text-ebay-red">S</span>
            <span className="text-ebay-blue">e</span>
            <span className="text-ebay-yellow">l</span>
            <span className="text-ebay-green">l</span>. Discover.
          </h2>
          <p className="text-xl md:text-2xl mb-8 text-gray-600">
            Millions of items at incredible prices, just for you
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Button
              size="lg"
              className="bg-ebay-blue hover:bg-blue-700 text-white text-lg px-8"
              onClick={() => navigate('/sell')}
            >
              Start Selling
            </Button>
            <Button
              size="lg"
              variant="outline"
              className="border-ebay-blue text-ebay-blue hover:bg-blue-100 hover:text-ebay-blue text-lg px-8"
              onClick={() => navigate('/marketplace')}
            >
              Shop Now
            </Button>
          </div>
        </div>
      </section>

      {/* Featured Items */}
      <section className="py-12 bg-gray-50 w-full">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h3 className="text-2xl font-bold text-gray-900 mb-8 text-center">
            {user ? 'Recommended for You' : 'Latest Listings'}
          </h3>
          {isLoading ? (
            <div className="flex justify-center">
              <Loader2 className="h-8 w-8 animate-spin" />
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              {featuredItems.map((item) => (
                <ItemCard key={item.item_id} item={item} />
              ))}
            </div>
          )}
        </div>
      </section>
      <footer className="bg-gray-800 text-white py-12 w-full">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
            <div>
              <h5 className="text-lg font-semibold mb-4">Buy</h5>

              <ul className="space-y-2 text-gray-300">
                <li>
                  <a href="#" className="hover:text-white">
                    Registration
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Bidding Help
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Stores
                  </a>
                </li>
              </ul>
            </div>

            <div>
              <h5 className="text-lg font-semibold mb-4">Sell</h5>

              <ul className="space-y-2 text-gray-300">
                <li>
                  <a href="#" className="hover:text-white">
                    Start Selling
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Seller Center
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Fees
                  </a>
                </li>
              </ul>
            </div>

            <div>
              <h5 className="text-lg font-semibold mb-4">Help</h5>

              <ul className="space-y-2 text-gray-300">
                <li>
                  <a href="#" className="hover:text-white">
                    Contact Us
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Safety Center
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Resolution Center
                  </a>
                </li>
              </ul>
            </div>

            <div>
              <h5 className="text-lg font-semibold mb-4">Community</h5>

              <ul className="space-y-2 text-gray-300">
                <li>
                  <a href="#" className="hover:text-white">
                    Forums
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Groups
                  </a>
                </li>

                <li>
                  <a href="#" className="hover:text-white">
                    Connect
                  </a>
                </li>
              </ul>
            </div>
          </div>

          <div className="border-t border-gray-700 mt-8 pt-8 text-center text-gray-400">
            <p>&copy; 2025 eBayish. All rights reserved.</p>
          </div>
        </div>
      </footer>
    </div>
  );
};
export default Landing;
