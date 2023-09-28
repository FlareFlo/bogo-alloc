# Bogo-alloc
## An allocator that makes C/C++ developers feel right at home.


---

Example usage:
```rust
// Note: 2^32 bytes are the limit for allocations on this heap
#[global_allocator]
static A: BogoAlloc<32> = BogoAlloc::new();
```