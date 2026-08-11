import React, { useState, useEffect, useMemo } from 'react';
import { toast } from 'sonner';
import { Loader2, Bell, CheckCircle, Trash2 } from 'lucide-react';
import { authFetch } from '@/utils/auth-fetch';
import { Notification, NotificationSummary } from '@/types/notification';
import NotificationList from '@/components/notifications/NotificationList';
import { Button } from '@/components/ui/button';
import { useWebSocketContext } from '@/contexts/WebSocketContext';

type NotificationStatusFilter = 'all' | 'unread' | 'read';

const Notifications: React.FC = () => {
  const { lastMessage } = useWebSocketContext();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [summary, setSummary] = useState<NotificationSummary | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [filter, setFilter] = useState<NotificationStatusFilter>('all');
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);
  const [isLoadingMore, setIsLoadingMore] = useState(false);

  const ITEMS_PER_PAGE = 20;

  // Fetch notification summary
  const fetchSummary = async () => {
    try {
      const response = await authFetch('/api/notifications/summary');
      if (response.ok) {
        const data = await response.json();
        setSummary(data);
      }
    } catch (error) {
      console.error('Error fetching notification summary:', error);
    }
  };

  // Fetch notifications with pagination
  const fetchNotifications = async (pageNum: number = 1, reset: boolean = false) => {
    try {
      if (pageNum === 1) {
        setIsLoading(true);
      } else {
        setIsLoadingMore(true);
      }

      const offset = (pageNum - 1) * ITEMS_PER_PAGE;
      const response = await authFetch(
        `/api/notifications?limit=${ITEMS_PER_PAGE}&offset=${offset}`
      );
      if (response.ok) {
        const data = await response.json();

        if (reset || pageNum === 1) {
          setNotifications(data);
        } else {
          setNotifications((prev) => [...prev, ...data]);
        }

        setHasMore(data.length === ITEMS_PER_PAGE);
        setPage(pageNum);
      }
    } catch (error) {
      console.error('Error fetching notifications:', error);
      toast.error('Failed to load notifications');
    } finally {
      setIsLoading(false);
      setIsLoadingMore(false);
    }
  };

  // Mark all as read
  const markAllAsRead = async () => {
    try {
      const response = await authFetch('/api/notifications/read-all', {
        method: 'PUT',
      });
      if (response.ok) {
        setNotifications((prev) =>
          prev.map((n) => ({ ...n, is_read: true, read_at: new Date().toISOString() }))
        );
        setSummary((prev) => (prev ? { ...prev, unread_count: 0 } : null));
        toast.success('All notifications marked as read');
      }
    } catch (error) {
      console.error('Error marking all as read:', error);
      toast.error('Failed to mark all as read');
    }
  };

  // Mark single notification as read
  const markAsRead = async (notificationId: string) => {
    try {
      const response = await authFetch(`/api/notifications/${notificationId}/read`, {
        method: 'PUT',
      });
      if (response.ok) {
        setNotifications((prev) =>
          prev.map((n) =>
            n.id === notificationId ? { ...n, is_read: true, read_at: new Date().toISOString() } : n
          )
        );
        setSummary((prev) =>
          prev ? { ...prev, unread_count: Math.max(0, prev.unread_count - 1) } : null
        );
      }
    } catch (error) {
      console.error('Error marking notification as read:', error);
      toast.error('Failed to mark as read');
    }
  };

  // Delete notification
  const deleteNotification = async (notificationId: string) => {
    try {
      const response = await authFetch(`/api/notifications/${notificationId}`, {
        method: 'DELETE',
      });
      if (response.ok) {
        const notification = notifications.find((n) => n.id === notificationId);
        const unreadDecrease = notification && !notification.is_read ? 1 : 0;

        setNotifications((prev) => prev.filter((n) => n.id !== notificationId));
        setSummary((prev) => {
          if (!prev) return null;
          return {
            ...prev,
            total_count: Math.max(0, prev.total_count - 1),
            unread_count: Math.max(0, prev.unread_count - unreadDecrease),
          };
        });
        toast.success('Notification deleted');
      }
    } catch (error) {
      console.error('Error deleting notification:', error);
      toast.error('Failed to delete notification');
    }
  };

  // Delete all notifications
  const deleteAllNotifications = async () => {
    if (
      !window.confirm(
        'Are you sure you want to delete all notifications? This action cannot be undone.'
      )
    ) {
      return;
    }

    try {
      const response = await authFetch('/api/notifications', {
        method: 'DELETE',
      });
      if (response.ok) {
        setNotifications([]);
        setSummary((prev) => (prev ? { ...prev, total_count: 0, unread_count: 0 } : null));
        toast.success('All notifications deleted');
      }
    } catch (error) {
      console.error('Error deleting all notifications:', error);
      toast.error('Failed to delete all notifications');
    }
  };

  // Filter notifications
  const filteredNotifications = useMemo(() => {
    let filtered = notifications;

    // Apply status filter
    if (filter === 'unread') {
      filtered = filtered.filter((n) => !n.is_read);
    } else if (filter === 'read') {
      filtered = filtered.filter((n) => n.is_read);
    }

    return filtered;
  }, [notifications, filter]);

  // Load initial data
  useEffect(() => {
    fetchSummary();
    fetchNotifications(1, true);
  }, []);

  // Listen for WebSocket notification events
  useEffect(() => {
    if (!lastMessage) return;

    if (lastMessage.type === 'NotificationReceived') {
      // const notificationData = lastMessage.data as NotificationReceivedPayload;

      // Don't show toast here - NotificationBell component handles toasts to avoid duplication

      // Update summary counts
      setSummary((prev) =>
        prev
          ? {
              ...prev,
              unread_count: prev.unread_count + 1,
              total_count: prev.total_count + 1,
            }
          : null
      );

      // Refresh notifications list to show new notification at top
      fetchNotifications(1, true);
    }
  }, [lastMessage]);

  // Load more notifications
  const loadMore = () => {
    if (!isLoadingMore && hasMore) {
      fetchNotifications(page + 1);
    }
  };

  const FilterButton: React.FC<{
    status: NotificationStatusFilter;
    label: string;
    count?: number;
    showRedCount?: boolean;
  }> = ({ status, label, count, showRedCount = false }) => (
    <Button
      variant={filter === status ? 'default' : 'outline'}
      onClick={() => setFilter(status)}
      className="bg-transparent border-none text-gray-600"
    >
      {label}
      {count !== undefined && count > 0 && showRedCount && (
        <span className="ml-1 px-1.5 py-0.5 bg-red-500 text-white text-xs rounded-full">
          {count > 99 ? '99+' : count}
        </span>
      )}
      {count !== undefined && count > 0 && !showRedCount && (
        <span className="ml-1 px-1.5 py-0.5 bg-gray-500 text-white text-xs rounded-full">
          {count > 99 ? '99+' : count}
        </span>
      )}
    </Button>
  );

  if (isLoading) {
    return (
      <div className="flex justify-center items-center h-64">
        <Loader2 className="h-12 w-12 animate-spin text-blue-600" />
      </div>
    );
  }

  const unreadCount = summary?.unread_count || 0;
  const totalCount = summary?.total_count || 0;

  return (
    <div className="container mx-auto p-4 md:p-6 space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-4xl font-bold text-gray-900">Notifications</h1>
          <p className="mt-2 text-lg text-gray-600">
            {totalCount === 0
              ? 'No notifications yet'
              : `${totalCount} total, ${unreadCount} unread`}
          </p>
        </div>

        {/* Actions */}
        {totalCount > 0 && (
          <div className="flex flex-wrap gap-2">
            {unreadCount > 0 && (
              <Button
                onClick={markAllAsRead}
                variant="outline"
                className="bg-transparent border-none text-gray-600"
              >
                <CheckCircle className="h-4 w-4 mr-2" />
                Mark all read
              </Button>
            )}
            <Button
              onClick={deleteAllNotifications}
              variant="outline"
              className="bg-transparent border-none text-red-600 hover:text-red-700"
            >
              <Trash2 className="h-4 w-4 mr-2" />
              Delete all
            </Button>
          </div>
        )}
      </div>

      {totalCount > 0 && (
        <>
          {/* Filters */}
          <div className="flex flex-wrap gap-2 justify-center">
            <FilterButton status="all" label="All" count={totalCount} showRedCount={false} />
            <FilterButton status="unread" label="Unread" count={unreadCount} showRedCount={true} />
            <FilterButton
              status="read"
              label="Read"
              count={totalCount - unreadCount}
              showRedCount={false}
            />
          </div>

          {/* Notifications List */}
          {filteredNotifications.length > 0 ? (
            <div className="bg-white border border-gray-200 rounded-lg shadow-sm">
              <NotificationList
                notifications={filteredNotifications}
                isLoading={false}
                onMarkAsRead={markAsRead}
                onDelete={deleteNotification}
              />

              {/* Load More Button */}
              {hasMore && filter === 'all' && (
                <div className="p-4 border-t border-gray-200 text-center">
                  <Button
                    onClick={loadMore}
                    disabled={isLoadingMore}
                    variant="outline"
                    className="bg-transparent border-none text-gray-600"
                  >
                    {isLoadingMore ? (
                      <>
                        <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                        Loading more...
                      </>
                    ) : (
                      'Load more notifications'
                    )}
                  </Button>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center py-16 border-2 border-dashed rounded-lg">
              <Bell className="mx-auto h-16 w-16 text-gray-400" />
              <h3 className="mt-4 text-xl font-semibold text-gray-800">No Notifications Found</h3>
              <p className="mt-1 text-gray-500">
                {filter === 'unread'
                  ? 'You have no unread notifications.'
                  : filter === 'read'
                    ? 'You have no read notifications.'
                    : 'No notifications match the selected filter.'}
              </p>
            </div>
          )}
        </>
      )}

      {totalCount === 0 && (
        <div className="text-center py-16 border-2 border-dashed rounded-lg">
          <Bell className="mx-auto h-16 w-16 text-gray-400" />
          <h3 className="mt-4 text-xl font-semibold text-gray-800">No Notifications</h3>
          <p className="mt-1 text-gray-500">
            You'll see updates about your items, bids, and auctions here.
          </p>
        </div>
      )}
    </div>
  );
};

export default Notifications;
