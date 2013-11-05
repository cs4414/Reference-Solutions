
RUSTC=rustc

all: zhtta

zhtta: zhtta.rs gash_mod lru_cache_mod
	rustc zhtta.rs

gash_mod: gash.rs
	rustc --lib gash.rs
    
lru_cache_mod: lru_cache.rs
	rustc --lib lru_cache.rs
	
clean:
	rm -fr *.so zhtta || true
