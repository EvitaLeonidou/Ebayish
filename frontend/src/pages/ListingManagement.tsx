import React, { useState, useEffect, useMemo } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Search,
  Eye,
  Edit,
  Trash2,
  Package,
  Loader2,
  User,
  Gavel,
  ShoppingBag,
} from 'lucide-react';
import { Item } from '@/types/item';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

const ListingManagement: React.FC = () => {
  const navigate = useNavigate();
  const [auctionListings, setAuctionListings] = useState<Item[]>([]);
  const [fixedPriceListings, setFixedPriceListings] = useState<Item[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [activeTab, setActiveTab] = useState<'auctions' | 'fixed_price'>('auctions');

  const statusOptions = [
    { value: 'all', label: 'All Statuses' },
    { value: 'active', label: 'Active' },
    { value: 'pending', label: 'Pending' },
    { value: 'sold', label: 'Sold' },
    { value: 'ended', label: 'Ended' },
  ];

  useEffect(() => {
    const fetchListings = async () => {
      setIsLoading(true);
      try {
        // Fetch auction listings
        const auctionResponse = await fetch(`/api/items?type=auction`);
        if (!auctionResponse.ok) throw new Error('Failed to fetch auction listings');
        const auctionData: Item[] = await auctionResponse.json();
        const transformedAuctions = auctionData.map((item: any) => ({
          ...item,
          images: item.images?.map((img: any) => img.url) || [],
        }));
        setAuctionListings(transformedAuctions);

        // Fetch fixed price listings
        const fixedPriceResponse = await fetch(`/api/items?type=fixed_price`);
        if (!fixedPriceResponse.ok) throw new Error('Failed to fetch fixed price listings');
        const fixedPriceData: Item[] = await fixedPriceResponse.json();
        const transformedFixedPrice = fixedPriceData.map((item: any) => ({
          ...item,
          images: item.images?.map((img: any) => img.url) || [],
        }));
        setFixedPriceListings(transformedFixedPrice);
      } catch (error) {
        toast.error('Could not load listings.');
      } finally {
        setIsLoading(false);
      }
    };

    fetchListings();
  }, []);

  const currentListings = activeTab === 'auctions' ? auctionListings : fixedPriceListings;

  const filteredListings = useMemo(() => {
    return currentListings.filter((listing) => {
      const search = searchTerm.toLowerCase();
      const matchesSearch = (listing.name?.toLowerCase() ?? '').includes(search);
      const matchesStatus = filterStatus === 'all' || listing.status === filterStatus;
      return matchesSearch && matchesStatus;
    });
  }, [currentListings, searchTerm, filterStatus]);

  const handleDelete = async (itemId: string) => {
    if (window.confirm('Are you sure you want to delete this listing? This cannot be undone.')) {
      try {
        const response = await fetch(`/api/items/${itemId}`, {
          method: 'DELETE',
        });
        if (!response.ok) {
          throw new Error('Failed to delete the listing.');
        }
        // Remove from both auction and fixed price listings
        setAuctionListings((prev) => prev.filter((item) => item.item_id !== itemId));
        setFixedPriceListings((prev) => prev.filter((item) => item.item_id !== itemId));
        toast.success('Listing deleted successfully.');
      } catch (error) {
        toast.error('An error occurred while deleting the listing.');
      }
    }
  };

  const getStatusBadge = (status?: string) => {
    if (!status) {
      return (
        <span className="px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
          Unknown
        </span>
      );
    }
    const statusStyles = {
      pending: 'bg-yellow-100 text-yellow-800',
      active: 'bg-green-100 text-green-800',
      sold: 'bg-blue-100 text-blue-800',
      ended: 'bg-gray-100 text-gray-800',
      rejected: 'bg-red-100 text-red-800',
    };
    const statusText = status.charAt(0).toUpperCase() + status.slice(1);
    return (
      <span
        className={`px-2 py-1 rounded-full text-xs font-medium ${
          statusStyles[status as keyof typeof statusStyles] || 'bg-gray-100 text-gray-800'
        }`}
      >
        {statusText}
      </span>
    );
  };

  const formatPrice = (price?: number) => {
    if (price === undefined || price === null) return 'N/A';
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">All Marketplace Listings</h1>

      {/* Tab Navigation */}
      <div className="flex space-x-1 bg-gray-100 p-1 rounded-lg">
        <button
          className={`flex-1 px-4 py-2 rounded-md font-medium transition-colors ${
            activeTab === 'auctions'
              ? 'bg-white text-blue-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900'
          }`}
          onClick={() => setActiveTab('auctions')}
        >
          <Gavel className="h-4 w-4 inline mr-2" />
          Auctions ({auctionListings.length})
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
          Fixed Price ({fixedPriceListings.length})
        </button>
      </div>

      <Card>
        <CardContent className="p-4">
          <div className="flex flex-col sm:flex-row gap-4">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
              <Input
                placeholder="Search by title..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10"
              />
            </div>
            <Select
              value={filterStatus}
              onValueChange={(value) => setFilterStatus(value as string)}
            >
              <SelectTrigger className="w-full sm:w-[180px]">
                <SelectValue placeholder="Filter by status..." />
              </SelectTrigger>
              <SelectContent>
                {statusOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
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
                  <th className="text-left py-3 px-4">Seller ID</th>
                  <th className="text-left py-3 px-4">Status</th>
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
                        <User className="h-4 w-4 text-gray-500" />
                        <span className="font-medium truncate" title={listing.seller_user_id}>
                          ...{listing.seller_user_id.slice(-8)}
                        </span>
                      </div>
                    </td>
                    <td className="py-4 px-4">{getStatusBadge(listing.status)}</td>
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
                          onClick={() => navigate(`/sell/edit/${listing.item_id}`)}
                        >
                          <Edit className="h-4 w-4" />
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
                {currentListings.length === 0
                  ? `There are no ${activeTab === 'auctions' ? 'auction' : 'fixed price'} listings.`
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
