import { create } from 'zustand';
import { NotificationItem } from '../types';
import { tauriApi } from '../lib/ipc';

interface NotificationStoreState {
  notifications: NotificationItem[];
  searchQuery: string;
  selectedPackage: string | null;
  filterOtpOnly: boolean;
  copiedOtp: string | null;
  copiedId: string | null;

  setSearchQuery: (query: string) => void;
  setSelectedPackage: (pkg: string | null) => void;
  setFilterOtpOnly: (val: boolean) => void;
  copyOtp: (code: string) => Promise<void>;
  copyNotificationText: (id: string, text: string) => Promise<void>;
  deleteNotification: (id: string) => Promise<void>;
  addNotification: (item: NotificationItem) => void;
  loadHistory: () => Promise<void>;
  clearHistory: () => Promise<void>;
  initNotificationListener: () => Promise<() => void>;
}

export const useNotificationStore = create<NotificationStoreState>((set, get) => ({
  notifications: [],
  searchQuery: '',
  selectedPackage: null,
  filterOtpOnly: false,
  copiedOtp: null,
  copiedId: null,

  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setSelectedPackage: (selectedPackage) => set({ selectedPackage }),
  setFilterOtpOnly: (filterOtpOnly) => set({ filterOtpOnly }),

  copyOtp: async (code: string) => {
    try {
      await tauriApi.copyOtpToClipboard(code, 45);
      set({ copiedOtp: code });
      setTimeout(() => {
        if (get().copiedOtp === code) {
          set({ copiedOtp: null });
        }
      }, 3000);
    } catch (e) {
      console.error('Failed to copy OTP:', e);
    }
  },

  copyNotificationText: async (id: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      set({ copiedId: id });
      setTimeout(() => {
        if (get().copiedId === id) {
          set({ copiedId: null });
        }
      }, 2000);
    } catch (e) {
      console.error('Failed to copy text:', e);
    }
  },

  deleteNotification: async (id: string) => {
    try {
      await tauriApi.deleteNotification(id);
      set((state) => ({
        notifications: state.notifications.filter((n) => n.id !== id),
      }));
    } catch (e) {
      console.error('Delete notification error:', e);
    }
  },

  addNotification: (item: NotificationItem) => {
    set((state) => {
      if (item.status === 'removed') {
        return {
          notifications: state.notifications.filter((n) => n.id !== item.id),
        };
      }

      // Check if already exists (update existing or prepend new)
      const existingIdx = state.notifications.findIndex((n) => n.id === item.id);
      if (existingIdx >= 0) {
        const updated = [...state.notifications];
        updated[existingIdx] = item;
        return { notifications: updated };
      } else {
        return { notifications: [item, ...state.notifications] };
      }
    });
  },

  loadHistory: async () => {
    try {
      const history = await tauriApi.getNotificationHistory(200);
      set({ notifications: history });
    } catch (e) {
      console.error('Failed to load history:', e);
    }
  },

  clearHistory: async () => {
    try {
      await tauriApi.clearNotificationHistory();
      set({ notifications: [] });
    } catch (e) {
      console.error('Failed to clear history:', e);
    }
  },

  initNotificationListener: async () => {
    await get().loadHistory();

    const unlisten = await tauriApi.onNotificationReceived((item) => {
      get().addNotification(item);
    });

    return unlisten;
  },
}));
