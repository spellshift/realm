/**
 * Utilities for sending system-level browser notifications using the Notification API.
 * These are used for URGENT priority alerts so the user receives an OS-level notification
 * (e.g. macOS notification center) even when the browser tab is not focused.
 */

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
}

/**
 * Show a system-level browser notification.
 * No-ops gracefully if the Notification API is unavailable or permission has not been granted.
 */
export const showSystemNotification = ({ title, body, onClick }: SystemNotificationOptions): void => {
    if (!('Notification' in window) || Notification.permission !== 'granted') {
        return;
    }

    const notification = new Notification(title, {
        body,
        icon: '/favicon.ico',
    });

    if (onClick) {
        notification.onclick = () => {
            window.focus();
            onClick();
            notification.close();
        };
    }
};
