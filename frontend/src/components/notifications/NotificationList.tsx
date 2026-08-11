import React from 'react';
import { formatDistanceToNow } from 'date-fns';
import {
  Gavel,
  DollarSign,
  ShoppingBag,
  TrendingUp,
  Trophy,
  X,
  CheckCircle,
  MessageCircle,
  Users,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Notification, NotificationType } from '@/types/notification';
import { useNavigate } from 'react-router-dom';

interface NotificationListProps {
  notifications: Notification[];
  isLoading: boolean;
  onMarkAsRead: (id: string) => void;
  onDelete: (id: string) => void;
}

const NotificationList: React.FC<NotificationListProps> = ({
  notifications,
  isLoading,
  onMarkAsRead,
  onDelete,
}) => {
  const navigate = useNavigate();

  const handleNotificationClick = (notification: Notification) => {
    // For message notifications, navigate to messaging page
    if (
      notification.notification_type === NotificationType.NEW_MESSAGE ||
      notification.notification_type === NotificationType.NEW_CHAT_ROOM
    ) {
      navigate('/messaging');
    }
    // For item/auction notifications, navigate to item detail page
    else if (
      notification.item_id &&
      (notification.notification_type === NotificationType.ITEM_SOLD ||
        notification.notification_type === NotificationType.NEW_BID ||
        notification.notification_type === NotificationType.AUCTION_ENDED ||
        notification.notification_type === NotificationType.BID_OUTBID ||
        notification.notification_type === NotificationType.AUCTION_WON ||
        notification.notification_type === NotificationType.AUCTION_LOST)
    ) {
      navigate(`/item/${notification.item_id}`);
    }

    // Mark as read when clicked
    if (!notification.is_read) {
      onMarkAsRead(notification.id);
    }
  };
  const getNotificationIcon = (type: NotificationType) => {
    switch (type) {
      case NotificationType.ITEM_SOLD:
        return <DollarSign className="h-5 w-5 text-green-600" />;
      case NotificationType.NEW_BID:
        return <Gavel className="h-5 w-5 text-blue-600" />;
      case NotificationType.AUCTION_ENDED:
        return <ShoppingBag className="h-5 w-5 text-purple-600" />;
      case NotificationType.BID_OUTBID:
        return <TrendingUp className="h-5 w-5 text-orange-600" />;
      case NotificationType.AUCTION_WON:
        return <Trophy className="h-5 w-5 text-yellow-600" />;
      case NotificationType.AUCTION_LOST:
        return <X className="h-5 w-5 text-gray-600" />;
      case NotificationType.NEW_CHAT_ROOM:
        return <Users className="h-5 w-5 text-blue-600" />;
      case NotificationType.NEW_MESSAGE:
        return <MessageCircle className="h-5 w-5 text-green-600" />;
      default:
        return <ShoppingBag className="h-5 w-5 text-gray-600" />;
    }
  };

  const getNotificationColor = (type: NotificationType) => {
    switch (type) {
      case NotificationType.ITEM_SOLD:
        return 'border-l-green-500';
      case NotificationType.NEW_BID:
        return 'border-l-blue-500';
      case NotificationType.AUCTION_ENDED:
        return 'border-l-purple-500';
      case NotificationType.BID_OUTBID:
        return 'border-l-orange-500';
      case NotificationType.AUCTION_WON:
        return 'border-l-yellow-500';
      case NotificationType.AUCTION_LOST:
        return 'border-l-gray-500';
      case NotificationType.NEW_CHAT_ROOM:
        return 'border-l-blue-500';
      case NotificationType.NEW_MESSAGE:
        return 'border-l-green-500';
      default:
        return 'border-l-gray-500';
    }
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (notifications.length === 0) {
    return (
      <div className="text-center p-8 text-gray-500">
        <ShoppingBag className="h-12 w-12 mx-auto mb-4 text-gray-300" />
        <p>No notifications yet</p>
        <p className="text-sm">You'll see updates about your items and bids here</p>
      </div>
    );
  }

  return (
    <div className="divide-y divide-gray-100">
      {notifications.map((notification) => (
        <div
          key={notification.id}
          className={`p-4 hover:bg-gray-50 transition-colors border-l-4 cursor-pointer ${
            notification.is_read ? 'opacity-75' : ''
          } ${getNotificationColor(notification.notification_type)}`}
          onClick={() => handleNotificationClick(notification)}
        >
          <div className="flex items-start justify-between">
            <div className="flex items-start space-x-3 flex-1">
              {/* Icon */}
              <div className="flex-shrink-0 mt-0.5">
                {getNotificationIcon(notification.notification_type)}
              </div>

              {/* Content */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center space-x-2">
                  <h4 className="text-sm font-medium text-gray-900 truncate">
                    {notification.title}
                  </h4>
                  {!notification.is_read && (
                    <div className="w-2 h-2 bg-blue-600 rounded-full flex-shrink-0"></div>
                  )}
                </div>

                <p className="text-sm text-gray-600 mt-1 line-clamp-2">{notification.message}</p>

                {notification.amount && (
                  <p className="text-sm font-medium text-green-600 mt-1">
                    {formatCurrency(notification.amount)}
                  </p>
                )}

                <p className="text-xs text-gray-500 mt-2">
                  {formatDistanceToNow(new Date(notification.created_at), {
                    addSuffix: true,
                  })}
                </p>
              </div>
            </div>

            {/* Actions */}
            <div className="flex items-center space-x-1 ml-2" onClick={(e) => e.stopPropagation()}>
              {!notification.is_read && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onMarkAsRead(notification.id)}
                  className="p-1 h-auto bg-transparent border-none text-gray-600"
                  title="Mark as read"
                >
                  <CheckCircle className="h-4 w-4 text-gray-400 hover:text-green-600" />
                </Button>
              )}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onDelete(notification.id)}
                className="p-1 h-auto bg-transparent border-none text-gray-600"
                title="Delete notification"
              >
                <X className="h-4 w-4 text-gray-400 hover:text-red-600" />
              </Button>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};

export default NotificationList;
