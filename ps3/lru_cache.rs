use std::hashmap::*;
use std::ptr;
use std::cast;
use std::to_str::ToStr;

struct LRUEntry<K, V> {
    key: Option<K>,
    value: Option<V>,
    size: uint,
    next: *mut LRUEntry<K, V>,
    prev: *mut LRUEntry<K, V>,
}

struct LRUCache<K, V> {
    map: HashMap<K, ~LRUEntry<K, V>>,
    //max_size: uint,
    free_size: uint,
    head: *mut LRUEntry<K, V>,
    tail: *mut LRUEntry<K, V>,
}

impl<K, V> LRUEntry<K, V> {
    pub fn new() -> LRUEntry<K, V> {
        LRUEntry { 
            key: None, 
            value: None,
            size: 0,
            next: ptr::mut_null(),
            prev: ptr::mut_null(),
        }
    }

    pub fn with_key_value(key: K, value: V, size: uint) -> LRUEntry<K, V> {
        LRUEntry {
            key: Some(key),
            value: Some(value),
            size: size,
            next: ptr::mut_null(),
            prev: ptr::mut_null(),
        }
    }
}

impl<A: ToStr, B: ToStr> ToStr for ~LRUEntry<A, B> {
    fn to_str(&self) -> ~str {
        format!("LRUEntry\\{ key: {}, value: {} \\}", self.key.to_str(), self.value.to_str())
    }
}

impl<K: Hash + Eq + Clone, V> LRUCache<K, V> {
    pub fn new(max_size: uint) -> LRUCache<K, V> {
        let cache = LRUCache {
            map: HashMap::new(),
            free_size: max_size,
            head: unsafe{ cast::transmute(~LRUEntry::<K, V>::new()) },
            tail: unsafe{ cast::transmute(~LRUEntry::<K, V>::new()) },
        };
        unsafe {
            (*cache.head).next = cache.tail;
            (*cache.tail).prev = cache.head;
        }
        return cache;
    }

    pub fn put(&mut self, key: K, value: V, size: uint) {
        let mut key_existed = false;
        let (node_ptr, node_opt) = match self.map.find_mut(&key) {
            Some(node) => {
                key_existed = true;
                node.value = Some(value);
                let node_ptr: &mut LRUEntry<K, V> = *node;
                let node_ptr: *mut LRUEntry<K, V> = node_ptr;
                (node_ptr, None)
            }
            None => {
                let mut node = ~LRUEntry::with_key_value(key.clone(), value, size);
                let node_ptr: *mut LRUEntry<K, V> = &mut *node;
                (node_ptr, Some(node))
            }
        };
        if key_existed {
            self.detach(node_ptr);
            self.attach(node_ptr);
        } else {
            while (self.free_size < size) {
                let lru = self.get_lru();
                self.detach(lru);
                unsafe {
                    match (*lru).key {
                        None => (),
                        Some(ref key) => { self.map.pop(key); }
                    }
                }
                
                let removed_size = unsafe {(*lru).size};
                self.free_size += removed_size;
            }
            self.map.swap(key, node_opt.unwrap());
            self.attach(node_ptr);
            self.free_size -= size;
        }
    }

    pub fn get<'a>(&'a mut self, key: &K) -> Option<&'a V> {
        let (value, node_ptr_opt) = match self.map.find_mut(key) {
            None => (None, None),
            Some(node) => {
                let node_ptr: &mut LRUEntry<K, V> = *node;
                let node_ptr: *mut LRUEntry<K, V> = unsafe{ cast::transmute(node_ptr) };
                unsafe {
                    match (*node_ptr).value {
                        None => (None, None),
                        Some(ref value) => (Some(value), Some(node_ptr))
                    }
                }
            }
        };
        match node_ptr_opt {
            None => (),
            Some(node_ptr) => {
                self.detach(node_ptr);
                self.attach(node_ptr);
            }
        }
        return value;
    }

    fn get_lru(&mut self) -> *mut LRUEntry<K, V> {
        unsafe { (*self.tail).prev }
    }

    fn detach(&mut self, node: *mut LRUEntry<K, V>) {
        unsafe {
            (*(*node).prev).next = (*node).next;
            (*(*node).next).prev = (*node).prev;
        }
    }

    fn attach(&mut self, node: *mut LRUEntry<K, V>) {
        unsafe {
            (*node).next = (*self.head).next;
            (*node).prev = self.head;
            (*self.head).next = node;
            (*(*node).next).prev = node;
        }
    }
}

#[unsafe_destructor]
impl<K,V> Drop for LRUCache<K,V> {
    fn drop(&mut self) {
        unsafe {
            let _: ~LRUEntry<K, V> = cast::transmute(self.head);
            let _: ~LRUEntry<K, V> = cast::transmute(self.tail);
        }
    }
}

fn main() {
    let mut cache: LRUCache<int, int> = LRUCache::new(4);
    cache.put(1, 10, 1);
    cache.put(2, 20, 1);
    cache.put(3, 30, 1);
    cache.put(4, 40, 1);
    cache.put(5, 50, 1);
    println("---Should be not found---");
    match cache.get(&1) {
        None => { println("Not found"); }
        Some(&num) => { println!("Found {}", num); }
    }
    println("---Entry with key 1 should be gone---");
    println(cache.map.to_str());

    cache.put(2, 40, 1);
    println("---Should find number 40---");
    match cache.get(&2) {
        None => { println("Not found"); }
        Some(&num) => { println!("Found {}", num); }
    }

    println("---Should still have entries with key 2, 3, 4, 5---");
    println(cache.map.to_str());
    cache.put(6, 60, 1);
    println("---Entry with key 3 should be gone now---");
    println(cache.map.to_str());
}
