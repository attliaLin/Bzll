use std::sync::{Arc, Mutex, Once, ONCE_INIT};
use std::{mem, thread};

#[derive(Clone)]
pub struct RenderManager {
    // Since we will be used in many threads, we need to protect
    // concurrent access
    pub inner: Arc<Mutex<u8>>
}

pub fn singleton() -> RenderManager {
    // Initialize it to a null value
    static mut SINGLETON: *const RenderManager = 0 as *const RenderManager;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            // Make it
            let singleton = RenderManager {
                inner: Arc::new(Mutex::new(0))
            };

            // Put it in the heap so it can outlive this call
            SINGLETON = mem::transmute(Box::new(singleton));
        });

        // Now we give out a copy of the data that is safe to use concurrently.
        (*SINGLETON).clone()
    }
}
