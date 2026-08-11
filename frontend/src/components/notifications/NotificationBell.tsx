import React, { useState, useEffect } from 'react';
import { Bell, X } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '@/contexts/AuthContext';
import { authFetch } from '@/utils/auth-fetch';
import { Notification, NotificationSummary } from '@/types/notification';
import NotificationList from './NotificationList';
import { useWebSocketContext } from '@/contexts/WebSocketContext';
import { AuctionEvent, NotificationReceivedPayload } from '@/types/websocket';
import { showNotificationToast } from '@/utils/notification-toast';

const NotificationBell: React.FC = () => {
  const { isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const { lastMessage } = useWebSocketContext();
  const [isOpen, setIsOpen] = useState(false);
  const [summary, setSummary] = useState<NotificationSummary | null>(null);
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Fetch notification summary
  const fetchSummary = async () => {
    if (!isAuthenticated) return;

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

  // Fetch recent notifications
  const fetchNotifications = async () => {
    if (!isAuthenticated) return;

    setIsLoading(true);
    try {
      const response = await authFetch('/api/notifications?limit=10');
      if (response.ok) {
        const data = await response.json();
        setNotifications(data);
      }
    } catch (error) {
      console.error('Error fetching notifications:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // Mark all as read
  const markAllAsRead = async () => {
    try {
      const response = await authFetch('/api/notifications/read-all', {
        method: 'PUT',
      });
      if (response.ok) {
        // Update local state
        setNotifications((prev) =>
          prev.map((n) => ({ ...n, is_read: true, read_at: new Date().toISOString() }))
        );
        setSummary((prev) => (prev ? { ...prev, unread_count: 0 } : null));
      }
    } catch (error) {
      console.error('Error marking all as read:', error);
    }
  };

  // Mark single notification as read
  const markAsRead = async (notificationId: string) => {
    try {
      const response = await authFetch(`/api/notifications/${notificationId}/read`, {
        method: 'PUT',
      });
      if (response.ok) {
        // Update local state
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
    }
  };

  // Delete notification
  const deleteNotification = async (notificationId: string) => {
    try {
      const response = await authFetch(`/api/notifications/${notificationId}`, {
        method: 'DELETE',
      });
      if (response.ok) {
        setNotifications((prev) => prev.filter((n) => n.id !== notificationId));
        setSummary((prev) => {
          if (!prev) return null;
          const notification = notifications.find((n) => n.id === notificationId);
          const unreadDecrease = notification && !notification.is_read ? 1 : 0;
          return {
            ...prev,
            total_count: Math.max(0, prev.total_count - 1),
            unread_count: Math.max(0, prev.unread_count - unreadDecrease),
          };
        });
      }
    } catch (error) {
      console.error('Error deleting notification:', error);
    }
  };

  // Fetch data when authenticated
  useEffect(() => {
    if (isAuthenticated) {
      fetchSummary();
    }
  }, [isAuthenticated]);

  // Fetch notifications when dropdown opens
  useEffect(() => {
    if (isOpen && isAuthenticated) {
      fetchNotifications();
    }
  }, [isOpen, isAuthenticated]);

  // Listen for WebSocket notification events
  useEffect(() => {
    if (!lastMessage || !isAuthenticated) return;

    if (lastMessage.type === AuctionEvent.NotificationReceived) {
      const notificationData = lastMessage.data as NotificationReceivedPayload;

      // Show clickable toast notification
      const isMessageNotification =
        notificationData.notification_type.includes('new_message') ||
        notificationData.notification_type.includes('new_chat_room');

      let navigationPath = '/notifications'; // Default fallback

      if (isMessageNotification && notificationData.item_id) {
        navigationPath = `/messaging/${notificationData.item_id}`;
      } else if (notificationData.item_id && !isMessageNotification) {
        // For item/auction notifications, navigate to item detail
        navigationPath = `/item/${notificationData.item_id}`;
      }

      showNotificationToast(notificationData, {
        onViewClick: () => navigate(navigationPath),
      });

      // Increment unread count in summary
      setSummary((prev) =>
        prev
          ? {
              ...prev,
              unread_count: prev.unread_count + 1,
              total_count: prev.total_count + 1,
            }
          : null
      );

      // If dropdown is open, refresh notifications list to show new notification
      if (isOpen) {
        fetchNotifications();
      }
    }
  }, [lastMessage, isAuthenticated, isOpen, navigate]);

  if (!isAuthenticated) {
    return null;
  }

  const unreadCount = summary?.unread_count || 0;

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="text-black hover:text-gray-600 relative px-1 py-1 transition-colors bg-transparent border-none"
      >
        <Bell className="h-4 w-4" />
        {unreadCount > 0 && (
          <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full h-3 w-3 flex items-center justify-center">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <>
          {/* Backdrop */}
          <div className="fixed inset-0 z-40" onClick={() => setIsOpen(false)} />

          {/* Dropdown */}
          <div className="absolute right-0 mt-2 w-96 bg-white border border-gray-200 rounded-lg shadow-lg z-50">
            {/* Header */}
            <div className="flex items-center justify-between p-4 border-b border-gray-200">
              <h3 className="text-lg font-semibold">Notifications</h3>
              <div className="flex items-center gap-2">
                {unreadCount > 0 && (
                  <button
                    onClick={markAllAsRead}
                    className="text-sm px-2 py-1 hover:bg-gray-100 rounded transition-colors bg-transparent border-none text-gray-600"
                  >
                    Mark all read
                  </button>
                )}
                <button
                  onClick={() => setIsOpen(false)}
                  className="p-1 hover:bg-gray-100 rounded transition-colors bg-transparent border-none text-gray-600"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
            </div>

            {/* Content */}
            <div className="max-h-96 overflow-y-auto">
              <NotificationList
                notifications={notifications}
                isLoading={isLoading}
                onMarkAsRead={markAsRead}
                onDelete={deleteNotification}
              />
            </div>

            {/* Footer */}
            {notifications.length > 0 && (
              <div className="p-3 border-t border-gray-200 text-center">
                <button
                  onClick={() => {
                    setIsOpen(false);
                    navigate('/notifications');
                  }}
                  className="text-blue-600 text-sm px-2 py-1 hover:bg-gray-50 rounded transition-colors bg-transparent border-none"
                >
                  View all notifications
                </button>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
};

export default NotificationBell;
