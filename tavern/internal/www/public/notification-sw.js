/**
 * Notification Service Worker
 *
 * A single service worker instance is shared across all browser tabs.
 * By routing system notifications through this worker, we guarantee that
 * only one OS-level notification is shown per event — regardless of how
 * many tabs are open.
 */

const shownTags = new Set();
const shownTagsOrder = [];
const MAX_TRACKED_TAGS = 500;

// Immediately take control of all client tabs on activation.
self.addEventListener('activate', (event) => {
    event.waitUntil(self.clients.claim());
});

/**
 * Listen for SHOW_NOTIFICATION messages from client tabs.
 * Deduplicates by tag so only the first tab to request a notification wins.
 */
self.addEventListener('message', (event) => {
    if (!event.data || event.data.type !== 'SHOW_NOTIFICATION') {
        return;
    }

    const { tag, title, body, icon, url } = event.data;

    // Deduplicate: skip if we already showed a notification with this tag
    if (tag && shownTags.has(tag)) {
        return;
    }

    if (tag) {
        shownTags.add(tag);
        shownTagsOrder.push(tag);
        // Evict the oldest entry to prevent unbounded memory growth
        if (shownTagsOrder.length > MAX_TRACKED_TAGS) {
            const oldest = shownTagsOrder.shift();
            shownTags.delete(oldest);
        }
    }

    self.registration.showNotification(title, {
        body,
        icon: icon || '/favicon.ico',
        tag: tag || undefined,
        data: { url },
    });
});

/**
 * Handle notification clicks — focus an existing tab and navigate,
 * or open a new window if no tabs exist.
 */
self.addEventListener('notificationclick', (event) => {
    event.notification.close();
    const url = event.notification.data?.url;

    event.waitUntil(
        self.clients
            .matchAll({ type: 'window', includeUncontrolled: true })
            .then((clientList) => {
                // Focus the first available client window
                for (const client of clientList) {
                    if ('focus' in client) {
                        return client.focus().then((focused) => {
                            if (url) {
                                focused.postMessage({ type: 'NOTIFICATION_CLICK', url });
                            }
                        });
                    }
                }
                // No existing window — open a new one
                return self.clients.openWindow(url || '/');
            })
    );
});
