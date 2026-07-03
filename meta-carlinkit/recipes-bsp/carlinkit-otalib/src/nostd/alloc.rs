use core::alloc::{GlobalAlloc, Layout};
use libc::{c_void, free, malloc, realloc};

struct LibcAlloc;

unsafe impl GlobalAlloc for LibcAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { malloc(layout.size()) as *mut u8 }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { free(ptr as *mut c_void) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { realloc(ptr as *mut c_void, new_size) as *mut u8 }
    }
}

#[global_allocator]
static A: LibcAlloc = LibcAlloc;

// use core::alloc::Layout as AllocLayout;

/*#[alloc_error_handler]
fn oom(_: AllocLayout) -> ! {
    unsafe { libc::_exit(1) }
}*/
