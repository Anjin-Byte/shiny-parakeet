use std::sync::atomic::{AtomicUsize, Ordering};

/// The inner storage for the double buffer.
struct Inner<T> {
    // Two buffers of type T.
    buffers: [T; 2],
    // An atomic index (0 or 1) indicating which buffer is currently the "front" (readable) one.
    front: AtomicUsize,
    // A simple reference count: initially 2 (one for the reader, one for the writer).
    ref_count: AtomicUsize,
}

/// The main double buffer type. We hide the details in `Inner<T>`.
pub struct DoubleBuffer<T> {
    inner: Box<Inner<T>>,
}

impl<T> DoubleBuffer<T> {
    /// Create a new double buffer with the two initial buffers.
    pub fn new(buffer0: T, buffer1: T) -> Self {
        Self {
            inner: Box::new(Inner {
                buffers: [buffer0, buffer1],
                front: AtomicUsize::new(0),         // initially, buffer at index 0 is front.
                ref_count: AtomicUsize::new(2),       // two handles will be created.
            }),
        }
    }

    /// Splits the double buffer into a reader and writer handle.
    /// This consumes the DoubleBuffer so that you can’t have overlapping handles.
    pub fn split(self) -> (DoubleBufferReader<T>, DoubleBufferWriter<T>) {
        // We use Box to ensure the inner allocation stays fixed.
        let boxed = self.inner;
        // Convert the Box into a raw pointer.
        let ptr = Box::into_raw(boxed);
        // Now create our two handles that share the same inner pointer.
        let reader = DoubleBufferReader { inner: ptr };
        let writer = DoubleBufferWriter { inner: ptr };
        (reader, writer)
    }
}

/// The reader handle – it only gives read-only access.
pub struct DoubleBufferReader<T> {
    inner: *mut Inner<T>,
}

/// The writer handle – it gives mutable access.
pub struct DoubleBufferWriter<T> {
    inner: *mut Inner<T>,
}

impl<T> DoubleBufferReader<T> {
    /// Returns an immutable reference to the current front buffer.
    pub fn read(&self) -> &T {
        // Safety: we know that the writer never gives us mutable access to the front buffer.
        unsafe { (*self.inner).front() }
    }
}

impl<T> DoubleBufferWriter<T> {
    /// Provides mutable access to the back buffer for writing.
    ///
    /// The provided closure `f` is given a mutable reference to the back buffer.
    /// After the closure finishes, you can call `swap()` to publish the changes.
    pub fn write<F: FnOnce(&mut T)>(&mut self, f: F) {
        unsafe {
            // Get mutable access to the back buffer.
            let back = (*self.inner).back_mut();
            f(back);
        }
    }

    /// Atomically swaps the roles of the buffers so that the back buffer (which was just updated)
    /// becomes the new front buffer.
    pub fn swap(&mut self) {
        unsafe {
            (*self.inner).swap();
        }
    }
}

// Implement helper methods on Inner<T>.
impl<T> Inner<T> {
    /// Returns a reference to the current front buffer.
    fn front(&self) -> &T {
        let index = self.front.load(Ordering::Acquire);
        &self.buffers[index]
    }

    /// Returns a mutable reference to the back buffer.
    fn back_mut(&mut self) -> &mut T {
        let front = self.front.load(Ordering::Relaxed);
        // Since there are only two buffers, the other index is `1 - front`.
        &mut self.buffers[1 - front]
    }

    /// Swaps the front index so that the back buffer becomes the front.
    fn swap(&mut self) {
        let current = self.front.load(Ordering::Relaxed);
        self.front.store(1 - current, Ordering::Release);
    }
}

unsafe impl<T> Send for DoubleBufferReader<T> {}
unsafe impl<T> Send for DoubleBufferWriter<T> {}

/// When either handle is dropped, we decrement the reference count. The last one to drop
/// will reconstruct the Box and free the inner memory.
impl<T> Drop for DoubleBufferReader<T> {
    fn drop(&mut self) {
        unsafe {
            // Atomically decrement ref_count. If the previous count was 1, this is the last handle.
            if (*self.inner).ref_count.fetch_sub(1, Ordering::AcqRel) == 1 {
                // Reconstruct the Box so that it will be dropped and free the allocation.
                let _ = Box::from_raw(self.inner);
            }
        }
    }
}

impl<T> Drop for DoubleBufferWriter<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.inner).ref_count.fetch_sub(1, Ordering::AcqRel) == 1 {
                let _ = Box::from_raw(self.inner);
            }
        }
    }
}

// Optionally, you might also implement Debug for these types.
impl<T: std::fmt::Debug> std::fmt::Debug for DoubleBufferReader<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoubleBufferReader")
            .field("front", unsafe { &(*self.inner).front() })
            .finish()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for DoubleBufferWriter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let front = unsafe { (*self.inner).front.load(Ordering::Acquire) };
        f.debug_struct("DoubleBufferWriter")
            .field("current_front_index", &front)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple test to check basic functionality.
    #[test]
    fn test_basic_usage() {
        // Create a double buffer with two integers.
        let db = DoubleBuffer::new(10, 20);
        let (reader, mut writer) = db.split();

        // Initially, front buffer is index 0 (i.e. value 10)
        assert_eq!(*reader.read(), 10);

        // Use the writer to update the back buffer.
        writer.write(|back| {
            *back = 30;
        });
        // Now swap so that the back becomes front.
        writer.swap();
        // The reader should now see the updated value.
        assert_eq!(*reader.read(), 30);

        // Now update again.
        writer.write(|back| {
            *back = 40;
        });
        writer.swap();
        assert_eq!(*reader.read(), 40);
    }

    // Test that swapping twice returns the original value.
    #[test]
    fn test_double_swap() {
        let db = DoubleBuffer::new(100, 200);
        let (reader, mut writer) = db.split();
        // Swap twice: front should eventually be the original front.
        writer.swap();
        writer.swap();
        assert_eq!(*reader.read(), 100);
    }

    // A stress test that simulates many writer updates and reader checks.
    #[test]
    fn test_stress() {
        let db = DoubleBuffer::new(0usize, 0usize);
        let (reader, mut writer) = db.split();

        let iterations = 10000;
        for i in 0..iterations {
            writer.write(|back| {
                *back = i;
            });
            writer.swap();
            let current = *reader.read();
            // The reader should see a value that is at most i
            // (if a swap happened later, it might be less than i, but never greater).
            assert!(current <= i);
        }
        // At the end, the front should equal iterations - 1.
        assert_eq!(*reader.read(), iterations - 1);
    }
}
