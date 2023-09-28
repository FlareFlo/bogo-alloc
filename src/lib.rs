use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::Mutex;
use nanorand::{Rng, WyRand};

const ORD: Ordering = Ordering::Relaxed;

/// Returns a totally memory safe allocator, with a fixed size heap-capacity
pub struct BogoAlloc<const SIZE: usize> {
	start: AtomicIsize,
	size: AtomicUsize,
	uninit: AtomicBool,
	rand: Mutex<Option<WyRand>>,
}

impl<const SIZE: usize> BogoAlloc<SIZE> {
	/// Creates a new uninitialized Allocator, which initializes itself when first used
	pub const fn new() -> Self {
		Self {
			start: AtomicIsize::new(0),
			size: AtomicUsize::new(0),
			uninit: AtomicBool::new(true),
			rand: Mutex::new(None),
		}
	}
	/// Returns random integer from internal RNG
	unsafe fn rand(&self) -> isize {
		isize::from_le_bytes(self.rand.lock().unwrap().as_mut().unwrap_unchecked().rand())
	}
	/// Returns random address from within range
	unsafe fn rand_addr(&self) -> isize {
		self.rand().abs() % self.size.load(ORD) as isize
	}
}

unsafe impl<const SIZE: usize> GlobalAlloc for BogoAlloc<SIZE> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		// Only one call should allocate, the rest gets deferred and waits for this to complete
		if self.uninit.swap(false, ORD) {
			let size: usize = 2_usize.pow(SIZE as u32);
			self.size.store(size, ORD);
			*self.rand.lock().unwrap() = Some(WyRand::new());
			self.start.store(libc::malloc(size.into()) as isize, ORD);
		} else {
			while self.start.load(ORD) == 0 {}
		}
		let align = layout.align() as isize;
		let mut offset = self.rand_addr().saturating_sub(layout.size() as isize); // To ensure allocations end up within bounds
		offset = (offset + align - 1) & !(align - 1); // Ensures the memory is aligned
		(self.start.load(ORD) + offset) as *mut u8
	}

	unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
		// :)
	}
}


#[cfg(test)]
mod tests {
	use crate::BogoAlloc;

	#[global_allocator]
	static A: BogoAlloc<32> = BogoAlloc::new();

	#[test]
	fn funny_values_everywhere() {
		let vecs = (1..42).map(|i| vec![i; 1500]).collect::<Vec<_>>();
		println!("{:?}", vecs);
	}
}