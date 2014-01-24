
RUSTC=rustc

all: zhtta

zhtta: zhtta.rs gash_mod lru_cache_mod
	$(RUSTC) zhtta.rs

gash_mod: gash.rs
	$(RUSTC) --lib gash.rs
    
lru_cache_mod: lru_cache.rs
	$(RUSTC) --lib lru_cache.rs
	
clean:
	rm -fr *.so zhtta || true
