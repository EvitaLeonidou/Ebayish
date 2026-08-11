import { toast } from 'sonner';
import { NotificationReceivedPayload } from '@/types/websocket';

/**
 * Shows an instant popup notification with appropriate styling based on type
 */
export const showNotificationToast = (
  notificationData: NotificationReceivedPayload,
  options?: {
    onViewClick?: () => void;
  }
) => {
  // Determine button label based on notification type
  const getButtonLabel = (notificationType: string) => {
    if (notificationType.includes('new_message') || notificationType.includes('new_chat_room')) {
      return 'Open Chat';
    }
    return 'View Item';
  };

  const toastOptions = {
    description: notificationData.message,
    duration: 5000,
    ...(options?.onViewClick && {
      action: {
        label: getButtonLabel(notificationData.notification_type),
        onClick: options.onViewClick,
      },
    }),
  };

  // Different toast styles based on notification type
  const notificationType = notificationData.notification_type;

  if (notificationType.includes('item_sold') || notificationType.includes('auction_won')) {
    toast.success(notificationData.title, toastOptions);
  } else if (notificationType.includes('bid_outbid') || notificationType.includes('auction_lost')) {
    toast.warning(notificationData.title, toastOptions);
  } else if (notificationType.includes('new_bid') || notificationType.includes('auction_ended')) {
    toast.info(notificationData.title, toastOptions);
  } else {
    toast(notificationData.title, toastOptions);
  }
};
