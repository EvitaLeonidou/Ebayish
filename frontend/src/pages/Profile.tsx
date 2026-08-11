import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useAuth } from '@/contexts/AuthContext';
import {
  LogOut,
  ShoppingBag,
  Gavel,
  Camera,
  Loader2,
  Package,
  History,
  TrendingUp,
} from 'lucide-react';
import { toast } from 'sonner';
import { User } from '@/types/user';
import { Item } from '@/types/item';
import ItemCard from '@/components/items/ItemCard';
import { useWebSocketContext } from '@/contexts/WebSocketContext';
import { AuctionEvent } from '@/types/websocket';
import { globalEvents, EVENTS } from '@/utils/events';

type UserProfile = User & {
  stats: {
    itemsSold: number;
    activeAuctions: number;
    activeFixedPrice: number;
    itemsBought: number;
  };
  preferences: {
    emailNotifications: boolean;
    bidAlerts: boolean;
    shippingAddress: string;
  };
  memberSince: string;
};

const Profile: React.FC = () => {
  const navigate = useNavigate();
  const { user, logout, token } = useAuth();
  const { lastMessage, subscribe, unsubscribe } = useWebSocketContext();
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<
    'overview' | 'bought' | 'sold' | 'auctions' | 'fixed_price' | 'bids' | 'security'
  >('overview');
  const [profilePicture, setProfilePicture] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [itemsBought, setItemsBought] = useState<Item[]>([]);
  const [itemsSold, setItemsSold] = useState<Item[]>([]);
  const [activeAuctions, setActiveAuctions] = useState<Item[]>([]);
  const [activeFixedPrice, setActiveFixedPrice] = useState<Item[]>([]);
  const [bidHistory, setBidHistory] = useState<any[]>([]);

  useEffect(() => {
    if (!user) {
      navigate('/login');
      return;
    }

    const fetchProfileData = async () => {
      setIsLoading(true);
      try {
        if (!token) {
          console.warn('No authentication token available');
          throw new Error('Authentication required');
        }

        const headers: Record<string, string> = {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        };

        console.log('Fetching user stats from:', `/api/users/${user.id}/stats`);
        console.log('With headers:', headers);

        const statsResponse = await fetch(`/api/users/${user.id}/stats`, { headers });

        console.log('Stats response status:', statsResponse.status);

        if (!statsResponse.ok) {
          const errorText = await statsResponse.text();
          console.error('Stats API error:', errorText);
          throw new Error(`Failed to fetch user stats: ${statsResponse.status} - ${errorText}`);
        }

        const statsData = await statsResponse.json();
        console.log('Received stats data:', statsData);

        // Ensure stats data has the expected structure
        const processedStats = {
          itemsSold: Number(statsData.itemsSold || statsData.items_sold || 0),
          activeAuctions: Number(statsData.activeAuctions || statsData.active_auctions || 0),
          activeFixedPrice: Number(statsData.activeFixedPrice || statsData.active_fixed_price || 0),
          itemsBought: Number(statsData.successfulBids || statsData.successful_bids || 0),
        };

        console.log('Processed stats:', processedStats);

        setProfile({
          ...user,
          stats: processedStats,
          preferences: {
            emailNotifications: false,
            bidAlerts: false,
            shippingAddress: 'Not available',
          },
          memberSince: user.created_at || new Date().toISOString(),
        });
      } catch (error) {
        console.error('Error fetching profile data:', error);
        toast.error('Could not load detailed profile. Displaying basic info.');
        // Fallback to basic info with default values
        setProfile({
          ...user,
          stats: { itemsSold: 0, activeAuctions: 0, activeFixedPrice: 0, itemsBought: 0 },
          preferences: {
            emailNotifications: false,
            bidAlerts: false,
            shippingAddress: 'Not available',
          },
          memberSince: user.created_at || new Date().toISOString(),
        });
      } finally {
        setIsLoading(false);
      }
    };

    fetchProfileData();
  }, [user, navigate, token]);

  // Handle real-time WebSocket updates
  useEffect(() => {
    if (!lastMessage || !user) return;

    const { type, data } = lastMessage;

    // Handle bid updates on user's active auctions
    if (type === AuctionEvent.BidPlaced && activeTab === 'auctions') {
      setActiveAuctions((prevAuctions) =>
        prevAuctions.map((auction) =>
          auction.item_id === data.item_id
            ? {
                ...auction,
                currently: Number(data.current_price),
                number_of_bids: data.bid_count,
              }
            : auction
        )
      );

      // Show notification for bids on user's auctions
      const userAuction = activeAuctions.find((item) => item.item_id === data.item_id);
      if (userAuction) {
        toast.info(`New bid of $${Number(data.current_price).toFixed(2)} on "${userAuction.name}"`);
      }
    }

    // Handle auction ended events
    if (type === AuctionEvent.AuctionEnded) {
      if (activeTab === 'auctions') {
        // Remove ended auction from active auctions or update status
        setActiveAuctions((prevAuctions) =>
          prevAuctions.map((auction) =>
            auction.item_id === data.item_id ? { ...auction, status: 'ended' as any } : auction
          )
        );
      }

      if (activeTab === 'bids') {
        // Update bid status based on auction result
        setBidHistory((prevBids) =>
          prevBids.map((bid) =>
            bid.item_id === data.item_id
              ? {
                  ...bid,
                  status: data.winner_username === user.username ? 'won' : 'lost',
                }
              : bid
          )
        );
      }
    }

    // Handle item sold events
    if (type === AuctionEvent.ItemSold) {
      // const payload = data as ItemSoldPayload;

      // If current user purchased the item, refresh bought items
      if (user.username && data.buyer_username === user.username) {
        if (activeTab === 'bought') {
          fetchUserActivity('bought');
        }
        toast.success(`Successfully purchased item!`);
      }

      // If current user sold the item, update their listings
      if (activeTab === 'auctions' || activeTab === 'fixed_price') {
        setActiveAuctions((prevAuctions) =>
          prevAuctions.map((auction) =>
            auction.item_id === data.item_id ? { ...auction, status: 'sold' as any } : auction
          )
        );
        setActiveFixedPrice((prevItems) =>
          prevItems.map((item) =>
            item.item_id === data.item_id ? { ...item, status: 'sold' as any } : item
          )
        );
      }
    }
  }, [lastMessage, user, activeTab, activeAuctions]);

  // Listen for purchase events to refresh data
  useEffect(() => {
    const handleItemPurchased = () => {
      // Refresh bought items if on that tab
      if (activeTab === 'bought') {
        fetchUserActivity('bought');
      }
    };

    globalEvents.on(EVENTS.ITEM_PURCHASED, handleItemPurchased);

    return () => {
      globalEvents.off(EVENTS.ITEM_PURCHASED, handleItemPurchased);
    };
  }, [activeTab, user]);

  // Cleanup WebSocket subscriptions when component unmounts or tab changes
  useEffect(() => {
    return () => {
      // Clean up any active subscriptions
      activeAuctions.forEach((item) => {
        if (item.item_id) {
          unsubscribe(item.item_id);
        }
      });

      activeFixedPrice.forEach((item) => {
        if (item.item_id) {
          unsubscribe(item.item_id);
        }
      });

      if (user?.id) {
        unsubscribe(`user_${user.id}_bids`);
      }
    };
  }, [activeTab, activeAuctions, activeFixedPrice, user, unsubscribe]);

  const fetchUserActivity = async (activityType: string) => {
    if (!user || !token) return;

    try {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      };

      let endpoint = '';
      switch (activityType) {
        case 'bought':
          endpoint = `/api/users/${user.id}/purchased-items`;
          break;
        case 'sold':
          endpoint = `/api/users/${user.id}/sold-items`;
          break;
        case 'auctions':
          endpoint = `/api/users/${user.id}/active-listings?type=auction`;
          break;
        case 'fixed_price':
          endpoint = `/api/users/${user.id}/active-listings?type=fixed_price`;
          break;
        case 'bids':
          endpoint = `/api/users/${user.id}/bid-history`;
          break;
        default:
          return;
      }

      console.log(`Fetching ${activityType} from ${endpoint}`);
      const response = await fetch(endpoint, { headers });

      if (!response.ok) {
        const errorText = await response.text();
        console.error(`API Error for ${activityType}:`, response.status, errorText);
        throw new Error(`Failed to fetch ${activityType}: ${response.status} - ${errorText}`);
      }

      const data = await response.json();
      console.log(`Received ${activityType} data:`, data);

      // Transform data for items (add image URL transformation)
      if (activityType !== 'bids') {
        const transformedData = data.map((item: any) => ({
          ...item,
          images: item.images?.map((img: any) => img.url) || [],
        }));

        switch (activityType) {
          case 'bought':
            setItemsBought(transformedData);
            break;
          case 'sold':
            setItemsSold(transformedData);
            break;
          case 'auctions':
            setActiveAuctions(transformedData);
            // Subscribe to WebSocket updates for active auctions
            transformedData.forEach((item: any) => {
              if (item.item_id) {
                subscribe(item.item_id);
              }
            });
            break;
          case 'fixed_price':
            setActiveFixedPrice(transformedData);
            // Subscribe to WebSocket updates for fixed price listings
            transformedData.forEach((item: any) => {
              if (item.item_id) {
                subscribe(item.item_id);
              }
            });
            break;
        }
      } else {
        setBidHistory(data);
        // Subscribe to user-specific bid updates
        if (user.id) {
          subscribe(`user_${user.id}_bids`);
        }
      }
    } catch (error) {
      console.error(`Error fetching ${activityType}:`, error);
      toast.error(`Failed to load ${activityType}.`);

      // Fallback to empty arrays
      switch (activityType) {
        case 'bought':
          setItemsBought([]);
          break;
        case 'sold':
          setItemsSold([]);
          break;
        case 'auctions':
          setActiveAuctions([]);
          break;
        case 'fixed_price':
          setActiveFixedPrice([]);
          break;
        case 'bids':
          setBidHistory([]);
          break;
      }
    }
  };

  const handlePictureUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      setProfilePicture(URL.createObjectURL(file));
      toast.info('Profile picture updated! Click "Save" to apply changes.');
    }
  };

  const handlePasswordChange = async (
    currentPassword: string,
    newPassword: string,
    confirmPassword: string
  ) => {
    if (!user || !token) {
      toast.error('Please log in to change your password.');
      return;
    }

    if (newPassword !== confirmPassword) {
      toast.error('New passwords do not match.');
      return;
    }

    if (newPassword.length < 6) {
      toast.error('New password must be at least 6 characters long.');
      return;
    }

    setIsSaving(true);
    try {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      };

      const response = await fetch(`/api/users/${user.id}/password`, {
        method: 'PUT',
        headers,
        body: JSON.stringify({
          currentPassword,
          newPassword,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.message || 'Failed to change password');
      }

      toast.success('Password changed successfully!');

      // Clear the form inputs
      const form = document.querySelector('form[data-password-form]') as HTMLFormElement;
      if (form) {
        form.reset();
      }
    } catch (error) {
      console.error('Password change error:', error);
      toast.error(error instanceof Error ? error.message : 'Failed to change password');
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading || !profile) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  const StatCard: React.FC<{ title: string; value: number | string; icon: React.ReactNode }> = ({
    title,
    value,
    icon,
  }) => (
    <Card className="text-center">
      <CardContent className="p-4 flex flex-col items-center justify-center">
        <div className="h-10 w-10 text-blue-600 flex items-center justify-center">{icon}</div>
        <p className="text-2xl font-bold mt-2">{value}</p>
        <p className="text-sm text-gray-500">{title}</p>
      </CardContent>
    </Card>
  );

  return (
    <div>
      <div className="mb-10 text-center">
        <h1 className="text-4xl font-bold text-gray-900">My Dashboard</h1>
        <p className="mt-2 text-lg text-gray-600">
          Welcome back, <span className="text-blue-600">{profile.username}!</span>
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Left Sidebar: Profile Card */}
        <div className="lg:col-span-1">
          <Card>
            <CardContent className="p-6 text-center">
              <div className="relative w-32 h-32 mx-auto mb-4 group">
                <img
                  src={
                    profilePicture ||
                    `https://api.dicebear.com/8.x/initials/svg?seed=${profile.username}`
                  }
                  alt="Profile"
                  className="w-full h-full rounded-full object-cover border-4 border-white shadow-md"
                />
                <label
                  htmlFor="profile-picture-upload"
                  className="absolute inset-0 bg-black/50 rounded-full flex items-center justify-center text-white opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
                >
                  <Camera className="h-8 w-8" />
                  <input
                    id="profile-picture-upload"
                    type="file"
                    className="hidden"
                    accept="image/*"
                    onChange={handlePictureUpload}
                  />
                </label>
              </div>
              <h2 className="text-xl font-bold">@{profile.username}</h2>
              <p className="text-sm text-gray-500">{profile.email}</p>
              <p className="text-xs text-gray-400 mt-2">
                Member since {new Date(profile.memberSince).toLocaleDateString()}
              </p>
              <Button
                onClick={logout}
                variant="destructive"
                className="w-full mt-6 bg-red-600 hover:bg-red-700"
              >
                <LogOut className="h-4 w-4 mr-2" /> Log Out
              </Button>
            </CardContent>
          </Card>
        </div>

        {/* Right Content: Tabs */}
        <div className="lg:col-span-3">
          <Card>
            <CardHeader className="border-b">
              <div className="flex flex-wrap gap-2">
                <Button
                  variant={activeTab === 'overview' ? 'default' : 'outline'}
                  onClick={() => setActiveTab('overview')}
                >
                  Overview
                </Button>
                <Button
                  variant={activeTab === 'bought' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('bought');
                    fetchUserActivity('bought');
                  }}
                >
                  <Package className="h-4 w-4 mr-1" />
                  Items Bought
                </Button>
                <Button
                  variant={activeTab === 'sold' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('sold');
                    fetchUserActivity('sold');
                  }}
                >
                  <TrendingUp className="h-4 w-4 mr-1" />
                  Items Sold
                </Button>
                <Button
                  variant={activeTab === 'auctions' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('auctions');
                    fetchUserActivity('auctions');
                  }}
                >
                  <Gavel className="h-4 w-4 mr-1" />
                  Active Auctions
                </Button>
                <Button
                  variant={activeTab === 'fixed_price' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('fixed_price');
                    fetchUserActivity('fixed_price');
                  }}
                >
                  <Package className="h-4 w-4 mr-1" />
                  Fixed Price
                </Button>
                <Button
                  variant={activeTab === 'bids' ? 'default' : 'outline'}
                  onClick={() => {
                    setActiveTab('bids');
                    fetchUserActivity('bids');
                  }}
                >
                  <History className="h-4 w-4 mr-1" />
                  My Bids
                </Button>
                <Button
                  variant={activeTab === 'security' ? 'default' : 'outline'}
                  onClick={() => setActiveTab('security')}
                >
                  Security
                </Button>
              </div>
            </CardHeader>
            <CardContent className="pt-6">
              {activeTab === 'overview' && (
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <h3 className="text-lg font-semibold">Activity Summary</h3>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        console.log('Manually refreshing stats...');
                        const fetchProfileData = async () => {
                          try {
                            if (!token) {
                              throw new Error('Authentication required');
                            }

                            const headers = {
                              'Content-Type': 'application/json',
                              Authorization: `Bearer ${token}`,
                            };

                            console.log('Manual fetch - user ID:', user.id);
                            console.log('Manual fetch - token present:', !!token);

                            const response = await fetch(`/api/users/${user.id}/stats`, {
                              headers,
                            });
                            console.log('Manual fetch response:', response.status);

                            if (!response.ok) {
                              const errorText = await response.text();
                              console.error('Manual fetch error:', errorText);
                              throw new Error(`HTTP ${response.status}: ${errorText}`);
                            }

                            const data = await response.json();
                            console.log('Manual fetch data:', data);

                            const processedStats = {
                              itemsSold: Number(data.itemsSold || data.items_sold || 0),
                              activeAuctions: Number(
                                data.activeAuctions || data.active_auctions || 0
                              ),
                              activeFixedPrice: Number(
                                data.activeFixedPrice || data.active_fixed_price || 0
                              ),
                              itemsBought: Number(data.successfulBids || data.successful_bids || 0),
                            };

                            setProfile((prev) =>
                              prev ? { ...prev, stats: processedStats } : null
                            );
                            toast.success('Stats refreshed successfully!');
                          } catch (error) {
                            console.error('Manual fetch error:', error);
                            toast.error(
                              `Failed to refresh stats: ${error instanceof Error ? error.message : 'Unknown error'}`
                            );
                          }
                        };
                        fetchProfileData();
                      }}
                    >
                      Refresh Stats
                    </Button>
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <StatCard
                      title="Items Sold"
                      value={profile.stats.itemsSold}
                      icon={<ShoppingBag className="h-8 w-8" />}
                    />
                    <StatCard
                      title="Active Auctions"
                      value={profile.stats.activeAuctions}
                      icon={<Gavel className="h-8 w-8" />}
                    />
                    <StatCard
                      title="Active Fixed Price"
                      value={profile.stats.activeFixedPrice}
                      icon={<Package className="h-8 w-8" />}
                    />
                    <StatCard
                      title="Items Bought"
                      value={profile.stats.itemsBought}
                      icon={<ShoppingBag className="h-8 w-8" />}
                    />
                  </div>
                </div>
              )}

              {activeTab === 'bought' && (
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold">Items You've Purchased</h3>
                  {itemsBought.length > 0 ? (
                    <div className="space-y-3">
                      {itemsBought.map((item: any) => (
                        <ItemCard
                          key={item.item_id}
                          item={item}
                          context="purchased"
                          contextData={{
                            purchase_date: item.purchase_date,
                            purchase_price: item.purchase_price,
                            seller_username: item.seller_username,
                          }}
                        />
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-gray-500">
                      <Package className="h-16 w-16 mx-auto mb-4 opacity-50" />
                      <p>You haven't purchased any items yet.</p>
                      <p className="text-sm">Start bidding or buying to see your purchases here!</p>
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'sold' && (
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold">Items You've Sold</h3>
                  {itemsSold.length > 0 ? (
                    <div className="space-y-3">
                      {itemsSold.map((item: any) => (
                        <ItemCard
                          key={item.item_id}
                          item={item}
                          context="sold"
                          contextData={{
                            sold_date: item.sold_date,
                            final_price: item.final_price,
                            buyer_username: item.buyer_username,
                            buyer_id: item.buyer_id,
                          }}
                        />
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-gray-500">
                      <TrendingUp className="h-16 w-16 mx-auto mb-4 opacity-50" />
                      <p>You haven't sold any items yet.</p>
                      <p className="text-sm">Create your first listing to start selling!</p>
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'auctions' && (
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <h3 className="text-lg font-semibold">Your Active Auctions</h3>
                    <Button
                      onClick={() => navigate('/sell')}
                      className="bg-green-600 hover:bg-green-700"
                    >
                      Create New Listing
                    </Button>
                  </div>
                  {activeAuctions.length > 0 ? (
                    <div className="space-y-3">
                      {activeAuctions.map((item) => (
                        <ItemCard key={item.item_id} item={item} context="owner" />
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-gray-500">
                      <Gavel className="h-16 w-16 mx-auto mb-4 opacity-50" />
                      <p>You don't have any active auctions.</p>
                      <p className="text-sm">Create your first auction to start selling!</p>
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'fixed_price' && (
                <div className="space-y-4">
                  <div className="flex justify-between items-center">
                    <h3 className="text-lg font-semibold">Your Fixed Price Listings</h3>
                    <Button
                      onClick={() => navigate('/sell')}
                      className="bg-green-600 hover:bg-green-700"
                    >
                      Create New Listing
                    </Button>
                  </div>
                  {activeFixedPrice.length > 0 ? (
                    <div className="space-y-3">
                      {activeFixedPrice.map((item) => (
                        <ItemCard key={item.item_id} item={item} context="owner" />
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-gray-500">
                      <Package className="h-16 w-16 mx-auto mb-4 opacity-50" />
                      <p>You don't have any fixed price listings.</p>
                      <p className="text-sm">
                        Create your first fixed price listing to start selling!
                      </p>
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'bids' && (
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold">Your Bidding History</h3>
                  {bidHistory.length > 0 ? (
                    <div className="space-y-3">
                      {bidHistory.map((bid, index) => (
                        <Card key={index} className="p-4">
                          <div className="flex justify-between items-center">
                            <div>
                              <h4 className="font-semibold">{bid.item_title || 'Unknown Item'}</h4>
                              <p className="text-sm text-gray-600">
                                Bid: ${Number(bid.amount || 0).toFixed(2)}
                              </p>
                              <p className="text-xs text-gray-500">
                                {bid.created_at
                                  ? new Date(bid.created_at).toLocaleDateString()
                                  : 'Unknown date'}
                              </p>
                            </div>
                            <div className="text-right">
                              <span
                                className={`px-2 py-1 rounded text-xs font-medium ${
                                  bid.status === 'winning'
                                    ? 'bg-green-100 text-green-800'
                                    : bid.status === 'outbid'
                                      ? 'bg-yellow-100 text-yellow-800'
                                      : bid.status === 'won'
                                        ? 'bg-blue-100 text-blue-800'
                                        : 'bg-red-100 text-red-800'
                                }`}
                              >
                                {bid.status || 'Unknown'}
                              </span>
                            </div>
                          </div>
                        </Card>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 text-gray-500">
                      <History className="h-16 w-16 mx-auto mb-4 opacity-50" />
                      <p>You haven't placed any bids yet.</p>
                      <p className="text-sm">Start bidding on auctions to see your history here!</p>
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'security' && (
                <div className="space-y-6">
                  <h3 className="text-lg font-semibold">Account Security</h3>
                  <form
                    data-password-form
                    onSubmit={(e) => {
                      e.preventDefault();
                      const formData = new FormData(e.target as HTMLFormElement);
                      const currentPassword = formData.get('currentPassword') as string;
                      const newPassword = formData.get('newPassword') as string;
                      const confirmPassword = formData.get('confirmPassword') as string;
                      handlePasswordChange(currentPassword, newPassword, confirmPassword);
                    }}
                    className="space-y-4"
                  >
                    <div className="space-y-2">
                      <Label htmlFor="current-password">Current Password</Label>
                      <Input
                        id="current-password"
                        name="currentPassword"
                        type="password"
                        required
                        disabled={isSaving}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="new-password">New Password</Label>
                      <Input
                        id="new-password"
                        name="newPassword"
                        type="password"
                        required
                        minLength={6}
                        disabled={isSaving}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="confirm-password">Confirm New Password</Label>
                      <Input
                        id="confirm-password"
                        name="confirmPassword"
                        type="password"
                        required
                        minLength={6}
                        disabled={isSaving}
                      />
                    </div>
                    <Button type="submit" disabled={isSaving}>
                      {isSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                      Change Password
                    </Button>
                  </form>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default Profile;
