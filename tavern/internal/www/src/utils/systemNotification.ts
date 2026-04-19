/**
 * Utilities for sending system-level browser notifications.
 *
 * Notifications are dispatched through a dedicated service worker so that only
 * **one** OS-level notification is shown per event, regardless of how many
 * tabs the user has open.  The service worker uses the `tag` option of
 * `showNotification()` to collapse duplicate requests into a single alert.
 *
 * If the service worker is unavailable (e.g. insecure context) the module
 * falls back to the regular `new Notification()` API.
 */

/** Cached registration for the notification service worker. */
let swRegistration: ServiceWorkerRegistration | null = null;

/**
 * Register the lightweight notification service worker and request
 * notification permission.  Safe to call multiple times.
 */
export const initNotificationServiceWorker = async (): Promise<void> => {
    if (!('Notification' in window)) {
        return;
    }

    // Request permission (no-ops if already granted / denied).
    if (Notification.permission === 'default') {
        await Notification.requestPermission();
    }

    // Register the notification service worker if possible.
    if ('serviceWorker' in navigator && !swRegistration) {
        try {
            swRegistration = await navigator.serviceWorker.register('/notification-sw.js');
        } catch (err) {
            console.warn('Notification service worker registration failed; falling back to Notification API.', err);
        }
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
    /** Unique tag used to deduplicate notifications across tabs. */
    tag?: string;
    /** URL to navigate to when the notification is clicked (used by service worker). */
    url?: string;
    /** Optional callback when the notification is clicked (fallback when SW is unavailable). */
    onClick?: () => void;
}

/**
 * Show a system-level browser notification.
 *
 * When a service worker is active the notification is dispatched through it,
 * ensuring only one notification per `tag` regardless of how many tabs are
 * open.  Falls back to `new Notification()` otherwise.
 */
export const showSystemNotification = ({ title, body, tag, url, onClick }: SystemNotificationOptions): void => {
    if (!('Notification' in window) || Notification.permission !== 'granted') {
        return;
    }

    // Prefer the service worker path for cross-tab deduplication.
    const activeWorker = swRegistration?.active;
    if (activeWorker) {
        activeWorker.postMessage({
            type: 'SHOW_NOTIFICATION',
            title,
            body,
            tag: tag || `${title}-${body}`,
            url,
        });
        return;
    }

    // Fallback — no service worker available.
    const notification = new Notification(title, {
        body,
        icon: '/favicon.ico',
        tag: tag || `${title}-${body}`,
    });

    if (onClick) {
        notification.onclick = () => {
            window.focus();
            onClick();
            notification.close();
        };
    }
};

/**
 * Set up a listener for `NOTIFICATION_CLICK` messages from the service
 * worker.  Returns a cleanup function to remove the listener.
 */
export const onServiceWorkerNotificationClick = (callback: (url: string) => void): (() => void) => {
    if (!('serviceWorker' in navigator)) {
        return () => {};
    }

    const handler = (event: MessageEvent) => {
        if (event.data && event.data.type === 'NOTIFICATION_CLICK') {
            callback(event.data.url);
        }
    };

    navigator.serviceWorker.addEventListener('message', handler);
    return () => navigator.serviceWorker.removeEventListener('message', handler);
};
