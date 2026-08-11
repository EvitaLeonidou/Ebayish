// Simple event system for inter-component communication
class EventEmitter {
  private events: { [key: string]: Function[] } = {};

  on(event: string, callback: Function) {
    if (!this.events[event]) {
      this.events[event] = [];
    }
    this.events[event].push(callback);
  }

  off(event: string, callback: Function) {
    if (!this.events[event]) return;
    this.events[event] = this.events[event].filter((cb) => cb !== callback);
  }

  emit(event: string, ...args: any[]) {
    if (!this.events[event]) return;
    this.events[event].forEach((callback) => callback(...args));
  }
}

export const globalEvents = new EventEmitter();

// Event types
export const EVENTS = {
  ITEM_PURCHASED: 'item_purchased',
  PROFILE_REFRESH: 'profile_refresh',
} as const;
