/**
 * Utilities for sending system-level browser notifications using the Notification API.
 * These are used for URGENT priority alerts so the user receives an OS-level notification
 * (e.g. macOS notification center) even when the browser tab is not focused.
 *
 * Cross-tab deduplication ensures that only one system notification is shown per event,
 * regardless of how many tabs are open. This is achieved via localStorage coordination
 * and the Notification API's built-in `tag` property.
 */

const SHOWN_NOTIFICATIONS_KEY = 'realm_shown_system_notifications';
const MAX_STORED_IDS = 200;

/**
 * Attempt to claim a notification ID for this tab. Returns true if this tab
 * is the first to claim it (and should therefore show the notification).
 * Uses localStorage for cross-tab coordination.
 */
const claimSystemNotification = (tag: string): boolean => {
    try {
        const raw = localStorage.getItem(SHOWN_NOTIFICATIONS_KEY);
        const ids: string[] = raw ? JSON.parse(raw) : [];
        if (ids.includes(tag)) {
            return false;
        }
        ids.push(tag);
        // Trim oldest entries to prevent unbounded growth
        if (ids.length > MAX_STORED_IDS) {
            ids.splice(0, ids.length - MAX_STORED_IDS);
        }
        localStorage.setItem(SHOWN_NOTIFICATIONS_KEY, JSON.stringify(ids));
        return true;
    } catch {
        // If localStorage is unavailable, fall through to allow the notification
        return true;
    }
};

/**
 * Request permission to show system notifications.
 * Safe to call multiple times — exits early if already granted or denied.
 */
export const requestNotificationPermission = async (): Promise<NotificationPermission> => {
    if (!('Notification' in window)) {
        return 'denied';
    }
    if (Notification.permission === 'granted' || Notification.permission === 'denied') {
        return Notification.permission;
    }
    return Notification.requestPermission();
};

/**
 * Options for creating a system notification.
 */
export interface SystemNotificationOptions {
    title: string;
    body: string;
    /** Optional callback when the notification is clicked. */
    onClick?: () => void;
    /** Unique tag to deduplicate notifications across tabs. */
    tag?: string;
}

/**
 * Show a system-level browser notification.
 * No-ops gracefully if the Notification API is unavailable or permission has not been granted.
 *
 * When a `tag` is provided, cross-tab deduplication ensures only one notification is shown
 * regardless of how many tabs are open.
 */
export const showSystemNotification = ({ title, body, onClick, tag }: SystemNotificationOptions): void => {
    if (!('Notification' in window) || Notification.permission !== 'granted') {
        return;
    }

    // Cross-tab deduplication: skip if another tab already showed this notification
    if (tag && !claimSystemNotification(tag)) {
        return;
    }

    const notification = new Notification(title, {
        body,
        icon: '/favicon.ico',
        // The Notification API's tag property provides browser-level dedup as a safety net
        ...(tag ? { tag } : {}),
    });

    if (onClick) {
        notification.onclick = () => {
            window.focus();
            onClick();
            notification.close();
        };
    }
};
