import React, { useState, useEffect, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { toast } from 'sonner';
import { Loader2, Gavel, ShoppingBag } from 'lucide-react';
import { Item } from '@/types/item';
import { Category } from '@/types/category';
import CategoryFilter from '@/components/search/CategoryFilter';
import PriceRangeSlider from '@/components/search/PriceRangeSlider';
import LocationFilter from '@/components/search/LocationFilter';
import ItemGrid from '@/components/items/ItemGrid';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useAuth } from '@/contexts/AuthContext';

const Marketplace: React.FC = () => {
  const [auctionItems, setAuctionItems] = useState<Item[]>([]);
  const [fixedPriceItems, setFixedPriceItems] = useState<Item[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchParams, setSearchParams] = useSearchParams();
  const [activeTab, setActiveTab] = useState<'all' | 'auctions' | 'fixed_price'>('all');
  const { user } = useAuth();

  // Get filter values from URL
  // const searchTerm = searchParams.get('q') || '';
  const selectedCategory = searchParams.get('category');
  const minPrice = searchParams.get('minPrice') || '';
  const maxPrice = searchParams.get('maxPrice') || '';
  const selectedLocation = searchParams.get('location');
  const sortBy = searchParams.get('sortBy') || 'newest';

  useEffect(() => {
    const fetchCategoriesAndItems = async () => {
      setIsLoading(true);
      try {
        // Fetch categories (can be done once)
        const categoriesResponse = await fetch('/api/categories');
        if (!categoriesResponse.ok) throw new Error('Failed to fetch categories');
        const categoriesData = await categoriesResponse.json();
        setCategories(categoriesData);

        // Fetch auction items
        const auctionQuery = new URLSearchParams(searchParams.toString());
        auctionQuery.set('type', 'auction');
        console.log(
          'Marketplace: fetching auction items with URL:',
          `/api/items?${auctionQuery.toString()}`
        );
        const auctionResponse = await fetch(`/api/items?${auctionQuery.toString()}`);
        if (!auctionResponse.ok) throw new Error('Failed to fetch auction items');
        const auctionData = await auctionResponse.json();
        const auctionItemsWithImages = auctionData
          .filter((item: any) => {
            // Filter out sold/ended items
            if (item.status === 'sold' || item.status === 'ended') return false;
            // Filter out user's own listings
            if (user && item.seller_user_id === user.id) return false;
            return true;
          })
          .map((item: any) => ({
            ...item,
            images: item.images?.map((img: any) => img.url) || [],
          }));
        setAuctionItems(auctionItemsWithImages);

        // Fetch fixed price items
        const fixedPriceQuery = new URLSearchParams(searchParams.toString());
        fixedPriceQuery.set('type', 'fixed_price');
        console.log(
          'Marketplace: fetching fixed price items with URL:',
          `/api/items?${fixedPriceQuery.toString()}`
        );
        const fixedPriceResponse = await fetch(`/api/items?${fixedPriceQuery.toString()}`);
        if (!fixedPriceResponse.ok) throw new Error('Failed to fetch fixed price items');
        const fixedPriceData = await fixedPriceResponse.json();
        const fixedPriceItemsWithImages = fixedPriceData
          .filter((item: any) => {
            // Filter out sold/ended items
            if (item.status === 'sold' || item.status === 'ended') return false;
            // Filter out user's own listings
            if (user && item.seller_user_id === user.id) return false;
            return true;
          })
          .map((item: any) => ({
            ...item,
            images: item.images?.map((img: any) => img.url) || [],
          }));
        setFixedPriceItems(fixedPriceItemsWithImages);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'An unknown error occurred';
        setError(errorMessage);
        toast.error(`Error: ${errorMessage}`);
      } finally {
        setIsLoading(false);
      }
    };

    fetchCategoriesAndItems();
  }, [searchParams]);

  // Combine all items for the "All" tab
  const allItems = useMemo(() => {
    return [...auctionItems, ...fixedPriceItems].sort((a, b) => {
      // Sort by the sortBy parameter
      switch (sortBy) {
        case 'price_asc':
          return (a.price || 0) - (b.price || 0);
        case 'price_desc':
          return (b.price || 0) - (a.price || 0);
        case 'ending_soon':
          // Prioritize auction items with end dates
          if (a.listing_type === 'auction' && b.listing_type === 'fixed_price') return -1;
          if (a.listing_type === 'fixed_price' && b.listing_type === 'auction') return 1;
          if (a.ends && b.ends) {
            return new Date(a.ends).getTime() - new Date(b.ends).getTime();
          }
          return 0;
        case 'newest':
        default:
          return new Date(b.started).getTime() - new Date(a.started).getTime();
      }
    });
  }, [auctionItems, fixedPriceItems, sortBy]);

  const handleFilterChange = (key: string, value: string | null) => {
    console.log('handleFilterChange called with:', key, '=', value);
    setSearchParams((prev) => {
      if (value === null || value === '') {
        console.log('Deleting', key, 'from URL params');
        prev.delete(key);
      } else {
        console.log('Setting', key, '=', value, 'in URL params');
        prev.set(key, value);
      }
      return prev;
    });
  };

  // const handleSaveSearch = () => {
  //   if (!isAuthenticated) {
  //     toast.error('You must be logged in to save a search.');
  //     return;
  //   }
  //   const currentSearch = searchParams.toString();
  //   if (!currentSearch) {
  //     toast.info('There are no active filters to save.');
  //     return;
  //   }
  //   // In a real app, you would make an API call here:
  //   // POST /api/user/saved-searches, body: { query: currentSearch }
  //   toast.success(`Search saved: ${currentSearch}`);
  //   console.log('Saving search:', currentSearch);
  // };

  if (error) {
    return <div className="text-center text-red-500">Failed to load marketplace: {error}</div>;
  }

  return (
    <div className="container mx-auto p-4 md:p-6">
      <div className="text-center mb-10">
        <h1 className="text-4xl font-bold text-gray-900">Marketplace</h1>
        <p className="mt-2 text-lg text-gray-600">
          Discover and bid on items from around the world.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Sidebar for Filters */}
        <aside className="lg:col-span-1">
          <div className="sticky top-24 space-y-6">
            <CategoryFilter
              categories={categories}
              selectedCategory={selectedCategory ? Number(selectedCategory) : null}
              onSelectCategory={(id) => handleFilterChange('category', id ? id.toString() : null)}
            />
            <LocationFilter
              selectedLocation={selectedLocation}
              onLocationChange={(location) => handleFilterChange('location', location)}
              items={[...auctionItems, ...fixedPriceItems]}
            />
            <PriceRangeSlider
              minPrice={minPrice}
              maxPrice={maxPrice}
              onPriceChange={(min, max) => {
                console.log('PriceRangeSlider onPriceChange: setting both params at once');
                setSearchParams((prev) => {
                  if (min === null || min === '') {
                    prev.delete('minPrice');
                  } else {
                    prev.set('minPrice', min);
                  }
                  if (max === null || max === '') {
                    prev.delete('maxPrice');
                  } else {
                    prev.set('maxPrice', max);
                  }
                  return prev;
                });
              }}
              items={[...auctionItems, ...fixedPriceItems]}
            />
          </div>
        </aside>

        {/* Main Content for Items */}
        <main className="lg:col-span-3">
          {/* Tab Navigation */}
          <div className="flex space-x-1 bg-gray-100 p-1 rounded-lg mb-6">
            <button
              className={`flex-1 px-4 py-2 rounded-md font-medium transition-colors ${
                activeTab === 'all'
                  ? 'bg-white text-blue-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
              onClick={() => setActiveTab('all')}
            >
              All Items ({allItems.length})
            </button>
            <button
              className={`flex-1 px-4 py-2 rounded-md font-medium transition-colors ${
                activeTab === 'auctions'
                  ? 'bg-white text-blue-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
              onClick={() => setActiveTab('auctions')}
            >
              <Gavel className="h-4 w-4 inline mr-2" />
              Auctions ({auctionItems.length})
            </button>
            <button
              className={`flex-1 px-4 py-2 rounded-md font-medium transition-colors ${
                activeTab === 'fixed_price'
                  ? 'bg-white text-blue-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
              onClick={() => setActiveTab('fixed_price')}
            >
              <ShoppingBag className="h-4 w-4 inline mr-2" />
              Fixed Price ({fixedPriceItems.length})
            </button>
          </div>

          <div className="flex justify-between items-center mb-4">
            <p className="text-sm text-gray-600">
              Showing{' '}
              {activeTab === 'all'
                ? allItems.length
                : activeTab === 'auctions'
                  ? auctionItems.length
                  : fixedPriceItems.length}{' '}
              results
            </p>
            <Select value={sortBy} onValueChange={(value) => handleFilterChange('sortBy', value)}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="Sort by" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="newest">Newest First</SelectItem>
                <SelectItem value="price_asc">Price: Low to High</SelectItem>
                <SelectItem value="price_desc">Price: High to Low</SelectItem>
                <SelectItem value="ending_soon">Ending Soon</SelectItem>
              </SelectContent>
            </Select>
          </div>
          {isLoading ? (
            <div className="flex justify-center items-center h-64">
              <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
            </div>
          ) : (
            <ItemGrid
              items={
                activeTab === 'all'
                  ? allItems
                  : activeTab === 'auctions'
                    ? auctionItems
                    : fixedPriceItems
              }
            />
          )}
        </main>
      </div>
    </div>
  );
};

export default Marketplace;
