export enum AuctionEvent {
  BidPlaced = 'BidPlaced',
  AuctionEnded = 'AuctionEnded',
  CountdownUpdate = 'CountdownUpdate',
  AuctionStarted = 'AuctionStarted',
  ItemSold = 'ItemSold',
  NotificationReceived = 'NotificationReceived',
  NewMessage = 'NewMessage',
  MessageDeleted = 'MessageDeleted',
  ChatRoomCreated = 'ChatRoomCreated',
  MessageNotification = 'MessageNotification',
}

// CORRECTED: Matches the backend event payload
export interface BidPlacedPayload {
  item_id: string;
  bid_id: string;
  bidder_username: string;
  amount: number;
  current_price: number;
  bid_count: number;
  timestamp: string;
}

// CORRECTED: Matches the backend event payload
export interface AuctionEndedPayload {
  item_id: string;
  winner_username: string | null;
  winning_bid: number | null;
  timestamp: string;
}

export interface CountdownUpdatePayload {
  item_id: string;
  time_remaining_seconds: number;
  timestamp: string;
}

export interface AuctionStartedPayload {
  item_id: string;
  title: string;
  starting_price: number;
  ends_at: string;
  timestamp: string;
}

export interface ItemSoldPayload {
  item_id: string;
  buyer_username: string;
  timestamp: string;
}

export interface NotificationReceivedPayload {
  user_id: string;
  notification_id: string;
  title: string;
  message: string;
  notification_type: string;
  timestamp: string;
  item_id?: string; // Contains chat_room_id for message notifications
}

export interface NewMessagePayload {
  chat_room_id: string;
  message_id: string;
  sender_username: string;
  content: string;
  timestamp: string;
}

export interface MessageDeletedPayload {
  chat_room_id: string;
  message_id: string;
  deleted_by_username: string;
  timestamp: string;
}

export interface ChatRoomCreatedPayload {
  chat_room_id: string;
  other_user_id: string;
  other_username: string;
  timestamp: string;
}

export interface MessageNotificationPayload {
  user_id: string;
  chat_room_id: string;
  sender_username: string;
  preview: string;
  timestamp: string;
}

// This is the raw message structure from the backend
export interface WebSocketMessage {
  type: AuctionEvent;
  data: any; // The payload (e.g., BidPlacedPayload) will be in this 'data' field
}

export interface SubscriptionRequest {
  type: 'Subscribe' | 'Unsubscribe';
  data: {
    item_id: string;
  };
}
