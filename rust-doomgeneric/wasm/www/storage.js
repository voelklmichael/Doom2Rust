// What the page keeps between visits, in IndexedDB: the last WAD (store `wad`) and the files the
// game writes (store `files`, keyed by the game they belong to). It is used by the page and by the
// worker. Where IndexedDB is not available (a private window, blocked site data) nothing is kept:
// every function then reports "nothing there" instead of failing, and the game still runs.
const DB_NAME = 'doom';
let opened = null;

function open() {
  opened ??= new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, 1);
    request.onupgradeneeded = () => {
      request.result.createObjectStore('wad');
      request.result.createObjectStore('files');
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
  return opened;
}

// Runs `action` on `store` in one transaction; resolves to its result once the data is safe.
async function transact(store, mode, action) {
  try {
    const db = await open();
    return await new Promise((resolve, reject) => {
      const transaction = db.transaction(store, mode);
      const request = action(transaction.objectStore(store));
      transaction.oncomplete = () => resolve(request.result);
      transaction.onerror = transaction.onabort = () => reject(transaction.error);
    });
  } catch (error) {
    console.warn(`storage (${store}) is not available:`, error);
    return undefined;
  }
}

export const get = (store, key) => transact(store, 'readonly', (s) => s.get(key));
export const put = (store, key, value) => transact(store, 'readwrite', (s) => s.put(value, key));
export const remove = (store, key) => transact(store, 'readwrite', (s) => s.delete(key));

// All entries of `store` whose key starts with `prefix`, as [key, value] pairs.
export async function entries(store, prefix) {
  const range = IDBKeyRange.bound(prefix, prefix + '￿');
  const [keys, values] = await Promise.all([
    transact(store, 'readonly', (s) => s.getAllKeys(range)),
    transact(store, 'readonly', (s) => s.getAll(range)),
  ]);
  return (keys ?? []).map((key, i) => [key, values[i]]);
}
