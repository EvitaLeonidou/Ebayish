import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  Users,
  Package,
  DollarSign,
  Loader2,
  ListTree,
  CheckCircle,
  Wifi,
  WifiOff,
  Trophy,
  Download,
  Brain,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { useWebSocketContext } from '@/contexts/WebSocketContext';
import { authFetch } from '@/utils/auth-fetch';

interface DashboardStats {
  totalUsers: number;
  pendingUsers: number;
  activeAuctions: number;
  activeFixedPrice: number;
  totalRevenue: number;
  itemsSold: number;
}

// NEW: Interface for a single activity item
interface ActivityItem {
  id: string;
  activity_type:
    | 'user_registration'
    | 'new_listing'
    | 'new_bid'
    | 'purchase'
    | 'auction_win'
    | string;
  message: string;
  timestamp: string; // ISO 8601 date string
}

const AdminDashboard: React.FC = () => {
  const navigate = useNavigate();
  const { status: wsStatus } = useWebSocketContext();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [activity, setActivity] = useState<ActivityItem[]>([]); // New state for activity
  const [isLoading, setIsLoading] = useState(true);
  const [isActivityLoading, setIsActivityLoading] = useState(true); // Separate loading state
  const [isRetraining, setIsRetraining] = useState(false); // State for model retraining
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Fetch dashboard stats
    const fetchStats = async () => {
      try {
        const response = await authFetch('/api/admin/dashboard/stats');
        if (!response.ok) throw new Error('Failed to fetch dashboard statistics');
        const rawData = await response.json();
        setStats({
          totalUsers: rawData.total_users,
          pendingUsers: rawData.pending_users,
          activeAuctions: rawData.active_auctions,
          activeFixedPrice: rawData.active_fixed_price,
          totalRevenue: rawData.total_revenue,
          itemsSold: rawData.items_sold,
        });
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Could not load stats.';
        setError(msg); // Set a general error for the page
        toast.error(msg);
      } finally {
        setIsLoading(false);
      }
    };

    // Fetch recent activity
    const fetchActivity = async () => {
      try {
        console.log('Fetching activity data...');
        const response = await authFetch('/api/admin/dashboard/activity');
        console.log('Activity response status:', response.status);
        if (!response.ok) {
          const errorText = await response.text();
          console.error('Activity fetch error:', response.status, errorText);
          throw new Error(`Failed to fetch recent activity: ${response.status}`);
        }
        const data = await response.json();
        console.log('Activity data received:', data);
        setActivity(data.activities);
      } catch (err) {
        toast.error(err instanceof Error ? err.message : 'Could not load activity feed.');
      } finally {
        setIsActivityLoading(false);
      }
    };

    fetchStats();
    fetchActivity();
  }, []);

  const formatRevenue = (value: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(value);
  };

  const getActivityIcon = (type: ActivityItem['activity_type']) => {
    switch (type) {
      case 'user_registration':
        return <Users className="h-4 w-4 text-blue-500" />;
      case 'new_listing':
        return <Package className="h-4 w-4 text-green-500" />;
      case 'new_bid':
        return <DollarSign className="h-4 w-4 text-purple-500" />;
      case 'purchase':
        return <CheckCircle className="h-4 w-4 text-orange-500" />;
      case 'auction_win':
        return <Trophy className="h-4 w-4 text-yellow-500" />;
      default:
        return <CheckCircle className="h-4 w-4 text-gray-500" />;
    }
  };

  const formatRelativeTime = (isoString: string) => {
    const date = new Date(isoString);
    const now = new Date();
    const seconds = Math.round((now.getTime() - date.getTime()) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.round(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.round(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    return date.toLocaleDateString();
  };

  const handleExport = async (format: 'json' | 'xml') => {
    try {
      const response = await authFetch(`/api/admin/export?format=${format}`);
      if (!response.ok) throw new Error('Failed to export data');

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;

      const timestamp = new Date().toISOString().split('T')[0];
      link.download = `listings_export_${timestamp}.${format}`;

      document.body.appendChild(link);
      link.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(link);

      toast.success(`Listings exported successfully as ${format.toUpperCase()}`);
    } catch (error) {
      console.error('Export failed:', error);
      toast.error('Failed to export listings data');
    }
  };

  const handleRetrainModel = async () => {
    setIsRetraining(true);
    try {
      const response = await authFetch('/api/admin/retrain-model', {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error('Failed to retrain recommendation model');
      }

      toast.success(
        'Recommendation model retrained successfully! New recommendations will be available immediately.'
      );
    } catch (error) {
      console.error('Model retraining failed:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to retrain model');
    } finally {
      setIsRetraining(false);
    }
  };

  const quickActions = [
    { title: 'Manage Users', icon: Users, path: '/admin/users', color: 'text-ebay-blue' },
    { title: 'Manage Listings', icon: Package, path: '/admin/listings', color: 'text-ebay-green' },
    { title: 'Items Sold', icon: CheckCircle, path: '/admin/items-sold', color: 'text-green-600' },
    {
      title: 'Manage Categories',
      icon: ListTree,
      path: '/admin/categories',
      color: 'text-ebay-yellow',
    },
  ];

  const exportActions = [
    { title: 'Export JSON', format: 'json' as const, color: 'text-blue-600' },
    { title: 'Export XML', format: 'xml' as const, color: 'text-purple-600' },
  ];

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
      {/* WebSocket Status Indicator */}
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-2xl font-bold">Admin Dashboard</h1>
        <div className="flex items-center gap-2">
          {wsStatus === 'connected' ? (
            <>
              <Wifi className="h-4 w-4 text-green-500" />
              <span className="text-sm text-green-600">WebSocket Connected</span>
            </>
          ) : wsStatus === 'connecting' ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin text-yellow-500" />
              <span className="text-sm text-yellow-600">Connecting...</span>
            </>
          ) : (
            <>
              <WifiOff className="h-4 w-4 text-red-500" />
              <span className="text-sm text-red-600">WebSocket Disconnected</span>
            </>
          )}
        </div>
      </div>

      {/* Improve Recommendations Section */}
      <Card className="border-purple-200 bg-gradient-to-r from-purple-50 to-indigo-50">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-purple-700">
            <Brain className="h-6 w-6" />
            Improve Recommendations
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between">
            <div>
              <p className="text-sm text-gray-600 mb-2">
                Retrain the recommendation algorithm with the latest user interaction data to
                improve personalization.
              </p>
              <p className="text-xs text-gray-500">
                This will analyze purchases, bids, and category views to generate better
                recommendations for users.
              </p>
            </div>
            <Button
              onClick={handleRetrainModel}
              disabled={isRetraining}
              className="bg-purple-600 hover:bg-purple-700 text-white px-6 py-2 flex items-center gap-2"
            >
              {isRetraining ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Retraining...
                </>
              ) : (
                <>
                  <Brain className="h-4 w-4" />
                  Retrain Model
                </>
              )}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-6">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Users</p>
                <p className="text-2xl font-bold text-gray-900">{stats?.totalUsers ?? 'N/A'}</p>
              </div>
              <Users className="h-8 w-8 text-gray-400" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Pending Users</p>
                <p className="text-2xl font-bold text-yellow-600">{stats?.pendingUsers ?? 'N/A'}</p>
              </div>
              <Users className="h-8 w-8 text-yellow-500" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Auctions</p>
                <p className="text-2xl font-bold text-gray-900">{stats?.activeAuctions ?? 'N/A'}</p>
              </div>
              <Package className="h-8 w-8 text-blue-400" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Fixed Price</p>
                <p className="text-2xl font-bold text-gray-900">
                  {stats?.activeFixedPrice ?? 'N/A'}
                </p>
              </div>
              <Package className="h-8 w-8 text-green-400" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Revenue</p>
                <p className="text-2xl font-bold text-gray-900">
                  {formatRevenue(stats?.totalRevenue ?? 0)}
                </p>
              </div>
              <DollarSign className="h-8 w-8 text-gray-400" />
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column (no changes here) */}
        <div className="lg:col-span-1 space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Quick Actions</CardTitle>
            </CardHeader>
            <CardContent className="grid grid-cols-2 gap-4">
              {quickActions.map((action) => {
                const Icon = action.icon;
                return (
                  <Button
                    key={action.title}
                    variant="outline"
                    className="h-auto p-4 flex flex-col items-center gap-2 text-center"
                    onClick={() => navigate(action.path)}
                  >
                    <Icon className={`h-6 w-6 ${action.color}`} />
                    <span className="text-xs font-semibold">{action.title}</span>
                  </Button>
                );
              })}
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Pending Tasks</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">Users for review</span>
                  <span className="text-sm font-semibold text-yellow-600">
                    {stats?.pendingUsers ?? 0}
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Export Data</CardTitle>
            </CardHeader>
            <CardContent className="grid grid-cols-1 gap-3">
              {exportActions.map((action) => (
                <Button
                  key={action.title}
                  variant="outline"
                  className="h-auto p-3 flex items-center gap-3 text-left justify-start"
                  onClick={() => handleExport(action.format)}
                >
                  <Download className={`h-5 w-5 ${action.color}`} />
                  <div>
                    <span className="text-sm font-semibold">{action.title}</span>
                    <p className="text-xs text-gray-500">
                      Download all listings data as {action.format.toUpperCase()}
                    </p>
                  </div>
                </Button>
              ))}
            </CardContent>
          </Card>
        </div>

        {/* MODIFIED: Right Column for dynamic activity */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>Recent Activity</CardTitle>
            </CardHeader>
            <CardContent>
              {isActivityLoading ? (
                <div className="flex justify-center items-center py-8">
                  <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
                </div>
              ) : activity.length > 0 ? (
                <div className="space-y-4">
                  {activity.map((act) => (
                    <div
                      key={act.id}
                      className="flex items-start gap-3 p-2 rounded-lg hover:bg-gray-50"
                    >
                      <div className="flex-shrink-0 mt-1">{getActivityIcon(act.activity_type)}</div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm text-gray-900">{act.message}</p>
                        <p className="text-xs text-gray-500">{formatRelativeTime(act.timestamp)}</p>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-center text-sm text-gray-500 py-8">
                  No recent activity to display.
                </p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default AdminDashboard;
