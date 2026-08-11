import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Loader2, ArrowLeft, Package, Calendar, DollarSign, User } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { authFetch } from '@/utils/auth-fetch';

interface Sale {
  id: string;
  item_id: string;
  item_name: string;
  buyer_username: string;
  seller_username: string;
  sale_amount: number;
  sale_date: string;
  sale_type: string; // "purchase" or "auction"
}

interface SalesResponse {
  sales: Sale[];
  total_count: number;
}

const ItemsSold: React.FC = () => {
  const navigate = useNavigate();
  const [sales, setSales] = useState<Sale[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchSales = async () => {
      try {
        const response = await authFetch('/api/admin/purchases');
        if (!response.ok) throw new Error('Failed to fetch sales');
        const data: SalesResponse = await response.json();
        setSales(data.sales);
        setTotalCount(data.total_count);
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Could not load sales.';
        setError(msg);
        toast.error(msg);
      } finally {
        setIsLoading(false);
      }
    };

    fetchSales();
  }, []);

  const formatPrice = (price: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(price);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-[calc(100vh-12rem)]">
        <Loader2 className="h-16 w-16 animate-spin text-blue-600" />
      </div>
    );
  }

  if (error) {
    return <div className="text-center text-red-500 p-8 bg-red-50 rounded-md">{error}</div>;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-4">
        <Button
          variant="outline"
          size="sm"
          onClick={() => navigate('/admin')}
          className="flex items-center gap-2"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to Dashboard
        </Button>
        <div>
          <h1 className="text-2xl font-bold">Items Sold</h1>
          <p className="text-gray-600">
            View all completed sales - auctions and purchases ({totalCount} total)
          </p>
        </div>
      </div>

      {/* Sales List */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Package className="h-5 w-5" />
            All Sales
          </CardTitle>
        </CardHeader>
        <CardContent>
          {sales.length === 0 ? (
            <div className="text-center py-8">
              <Package className="h-12 w-12 mx-auto text-gray-400 mb-4" />
              <p className="text-gray-500">No sales found</p>
            </div>
          ) : (
            <div className="space-y-4">
              {sales.map((sale) => (
                <div
                  key={sale.id}
                  className="border rounded-lg p-4 hover:bg-gray-50 transition-colors"
                >
                  <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                    {/* Item Info */}
                    <div className="md:col-span-2">
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="font-semibold text-gray-900">{sale.item_name}</h3>
                        <span
                          className={`px-2 py-1 rounded-full text-xs font-medium ${
                            sale.sale_type === 'auction'
                              ? 'bg-purple-100 text-purple-800'
                              : 'bg-green-100 text-green-800'
                          }`}
                        >
                          {sale.sale_type === 'auction' ? 'Auction Win' : 'Direct Purchase'}
                        </span>
                      </div>
                      <p className="text-sm text-gray-500">Item ID: {sale.item_id}</p>
                    </div>

                    {/* Users */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2 text-sm">
                        <User className="h-4 w-4 text-green-600" />
                        <span className="text-gray-600">Buyer:</span>
                        <span className="font-medium">{sale.buyer_username}</span>
                      </div>
                      <div className="flex items-center gap-2 text-sm">
                        <User className="h-4 w-4 text-blue-600" />
                        <span className="text-gray-600">Seller:</span>
                        <span className="font-medium">{sale.seller_username}</span>
                      </div>
                    </div>

                    {/* Price & Date */}
                    <div className="space-y-2">
                      <div className="flex items-center gap-2 text-sm">
                        <DollarSign className="h-4 w-4 text-green-600" />
                        <span className="font-semibold text-green-700">
                          {formatPrice(sale.sale_amount)}
                        </span>
                      </div>
                      <div className="flex items-center gap-2 text-sm text-gray-600">
                        <Calendar className="h-4 w-4" />
                        {formatDate(sale.sale_date)}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default ItemsSold;
