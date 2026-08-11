import React, { useState, useEffect, useMemo } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Search, Eye, Trash2, Package, Loader2, User as UserIcon } from 'lucide-react';
import { Item } from '@/types/item';
import { User } from '@/types/user';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { authFetch } from '@/utils/auth-fetch';

const ListingManagement: React.FC = () => {
  const navigate = useNavigate();
  const [listings, setListings] = useState<Item[]>([]);
  const [users, setUsers] = useState<User[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    const fetchData = async () => {
      setIsLoading(true);
      try {
        // Fetch listings and users in parallel
        const [listingsResponse, usersResponse] = await Promise.all([
          fetch(`/api/items`),
          authFetch('/api/admin/users'),
        ]);

        if (!listingsResponse.ok) throw new Error('Failed to fetch listings');
        if (!usersResponse.ok) throw new Error('Failed to fetch users');

        const listingsData: Item[] = await listingsResponse.json();
        const usersData: User[] = await usersResponse.json();

        // Transform listings images
        const transformedListings = listingsData.map((item: any) => ({
          ...item,
          images: item.images?.map((img: any) => img.url) || [],
        }));
        setListings(transformedListings);
        setUsers(usersData);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : 'Could not load page data.');
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, []);

  // Create a map for quick user lookup
  const userMap = useMemo(() => {
    return users.reduce(
      (acc, user) => {
        acc[user.id] = user.username;
        return acc;
      },
      {} as Record<string, string>
    );
  }, [users]);

  const filteredListings = useMemo(() => {
    return listings.filter((listing) => {
      const search = searchTerm.toLowerCase();
      const sellerUsername = userMap[listing.seller_user_id]?.toLowerCase() || '';

      const matchesSearch =
        (listing.name?.toLowerCase() ?? '').includes(search) || sellerUsername.includes(search);

      // Filter to only show active auctions and fixed price items
      const isActive = listing.status === 'active';
      const isValidType =
        listing.listing_type === 'auction' || listing.listing_type === 'fixed_price';

      return matchesSearch && isActive && isValidType;
    });
  }, [listings, searchTerm, userMap]);

  const handleDelete = async (itemId: string) => {
    if (window.confirm('Are you sure you want to delete this listing? This cannot be undone.')) {
      try {
        const response = await fetch(`/api/items/${itemId}`, {
          method: 'DELETE',
        });
        if (!response.ok) {
          throw new Error('Failed to delete the listing.');
        }
        setListings((prev) => prev.filter((item) => item.item_id !== itemId));
        toast.success('Listing deleted successfully.');
      } catch (error) {
        toast.error('An error occurred while deleting the listing.');
      }
    }
  };

  const formatPrice = (price?: number) => {
    if (price === undefined || price === null) return 'N/A';
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
  };

  const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1).replace(/_/g, ' ');

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Active Listings Management</h1>
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-col sm:flex-row gap-4">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
              <Input
                placeholder="Search by title or seller..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-3 px-4">Listing</th>
                  <th className="text-left py-3 px-4">Seller</th>
                  <th className="text-left py-3 px-4">Type / Category</th>
                  <th className="text-left py-3 px-4">Price</th>
                  <th className="text-left py-3 px-4">Bids</th>
                  <th className="text-left py-3 px-4">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredListings.map((listing) => (
                  <tr key={listing.item_id} className="border-b hover:bg-gray-50">
                    <td className="py-4 px-4">
                      <div className="flex items-start gap-3">
                        <div className="w-12 h-12 bg-gray-200 rounded-md flex-shrink-0">
                          {listing.images?.[0] ? (
                            <img
                              src={listing.images[0]}
                              alt={listing.name}
                              className="w-full h-full object-cover rounded-md"
                            />
                          ) : (
                            <Package className="h-full w-full text-gray-400 p-2" />
                          )}
                        </div>
                        <div>
                          <p className="font-medium text-gray-900">{listing.name}</p>
                          <p className="text-gray-500 text-xs">ID: {listing.item_id}</p>
                        </div>
                      </div>
                    </td>
                    <td className="py-4 px-4">
                      <div className="flex items-center gap-2">
                        <UserIcon className="h-4 w-4 text-gray-500" />
                        <span
                          className="font-medium truncate"
                          title={userMap[listing.seller_user_id] || 'Unknown'}
                        >
                          {userMap[listing.seller_user_id] || 'Unknown Seller'}
                        </span>
                      </div>
                    </td>
                    <td className="py-4 px-4">
                      <div>
                        <p className="font-medium">{capitalize(listing.listing_type)}</p>
                        <p className="text-xs text-gray-500">
                          {listing.categories?.[0] || 'Uncategorized'}
                        </p>
                      </div>
                    </td>
                    <td className="py-4 px-4 font-medium">
                      {formatPrice(
                        listing.listing_type === 'auction' ? listing.currently : listing.price
                      )}
                    </td>
                    <td className="py-4 px-4">
                      {listing.listing_type === 'fixed_price' ? '-' : listing.number_of_bids || 0}
                    </td>
                    <td className="py-4 px-4">
                      <div className="flex items-center gap-1">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => navigate(`/item/${listing.item_id}`)}
                        >
                          <Eye className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          className="text-red-600 hover:text-red-700"
                          onClick={() => handleDelete(listing.item_id)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {filteredListings.length === 0 && (
            <div className="text-center py-8">
              <p className="text-gray-500">
                {listings.length === 0
                  ? 'There are no listings in the marketplace.'
                  : 'No listings match your criteria.'}
              </p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default ListingManagement;
