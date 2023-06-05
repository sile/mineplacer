const CACHE_NAME = 'mineplacer-0.1.0';

self.addEventListener('install', (e) => {
    console.log('[Service Worker] Install');
    e.waitUntil(self.skipWaiting());
});

self.addEventListener('fetch', (e) => {
    if(!e.request.url.startsWith('https://')){
        e.respondWith(fetch(e.request));
        return;
    }

    e.respondWith(
        caches.match(e.request).then((r) => {
            console.log('[Service Worker] Fetching resource: ' + e.request.url);
            return r || fetch(e.request).then((response) => {
                return caches.open(CACHE_NAME).then((cache) => {
                    console.log('[Service Worker] Caching new resource: ' + e.request.url);
                    cache.put(e.request, response.clone());
                    return response;
                });
            });
        })
    );
});

self.addEventListener('activate', (e) => {
    console.log('[Service Worker] Activate');
    e.waitUntil(
        caches.keys().then((keyList) => {
            return Promise.all(keyList.map((key) => {
                if(key !== CACHE_NAME) {
                    console.log(`[Service Worker] Delete old cache: ${key}`);
                    return caches.delete(key);
                }
            }));
        })
    );
});
