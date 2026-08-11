export interface Notification {
  id: string;
  user_id: string;
  notification_type: NotificationType;
  title: string;
  message: string;
  item_id?: string;
  related_user_id?: string;
  amount?: number;
  is_read: boolean;
  created_at: string;
  read_at?: string;
}

export enum NotificationType {
  ITEM_SOLD = 'item_sold',
  NEW_BID = 'new_bid',
  AUCTION_ENDED = 'auction_ended',
  BID_OUTBID = 'bid_outbid',
  AUCTION_WON = 'auction_won',
  AUCTION_LOST = 'auction_lost',
  NEW_CHAT_ROOM = 'new_chat_room',
  NEW_MESSAGE = 'new_message',
}

export interface NotificationSummary {
  total_count: number;
  unread_count: number;
  notification_types: NotificationTypeCount[];
}

export interface NotificationTypeCount {
  notification_type: NotificationType;
  count: number;
  unread_count: number;
}

export interface NotificationFilters {
  type?: NotificationType;
  is_read?: boolean;
  limit?: number;
  offset?: number;
}
