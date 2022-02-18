use std::{alloc, mem, ptr};

pub struct RingBuffer<T: Copy, const N: usize> {
    buffer: *const T,
    is_empty: bool,
    head: *mut T,
    tail: *mut T,
}

#[derive(Debug, PartialEq)]
pub enum RingBufferError<T> {
    InitializationLayoutError,
    InitializationAllocationError,
    OverflowError(T),
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Result<Self, RingBufferError<T>> {
        unsafe {
            let element_size = mem::size_of::<T>().checked_next_power_of_two();
            if element_size == None {
                return Err(RingBufferError::InitializationLayoutError);
            }

            let buffer_layout = alloc::Layout::from_size_align(N, element_size.unwrap())
                .map_err(|_| RingBufferError::InitializationLayoutError)?;
            let buffer = alloc::alloc_zeroed(buffer_layout) as *mut T;
            if buffer == ptr::null_mut() {
                return Err(RingBufferError::InitializationAllocationError);
            }
            Ok(Self {
                buffer,
                is_empty: true,
                head: buffer,
                tail: buffer,
            })
        }
    }

    pub fn next_value(&mut self) -> Option<T> {
        unsafe {
            if !self.is_empty {
                let value = *self.head;
                self.head = self.next_pointer_value(self.head) as *mut T;
                if self.head == self.tail {
                    self.is_empty = true;
                }
                Some(value)
            } else {
                None
            }
        }
    }

    pub fn push_value(&mut self, value: T) -> Result<(), RingBufferError<T>> {
        unsafe {
            if self.is_overflow() {
                Err(RingBufferError::OverflowError(value))
            } else {
                *self.tail = value;
                self.tail = self.next_pointer_value(self.tail) as *mut T;
                self.is_empty = false;
                Ok(())
            }
        }
    }

    fn is_overflow(&self) -> bool {
        !self.is_empty && (self.tail == self.head)
    }

    fn next_pointer_value(&self, ptr: *const T) -> *const T {
        unsafe {
            let buffer_end = self.buffer.offset(N as isize - 1);
            if ptr.offset(1) > buffer_end {
                self.buffer
            } else {
                ptr.offset(1)
            }
        }
    }

    fn free_buffer(&mut self) {
        unsafe {
            let buffer_layout =
                alloc::Layout::from_size_align(N, mem::size_of::<T>().next_power_of_two()).unwrap();
            alloc::dealloc(self.buffer as *mut u8, buffer_layout);
        }
    }
}

impl<T: Copy, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        self.free_buffer()
    }
}

#[cfg(test)]
mod test {
    use crate::ring_buffer::RingBufferError;

    use super::RingBuffer;

    #[test]
    fn test_should_get_four_pushed_values_in_same_order() {
        let mut buff = RingBuffer::<u8, 1024>::new()
            .expect("Allocation should be successful in this test case");
        || -> Result<(), RingBufferError<_>> {
            buff.push_value(32)?;
            buff.push_value(2)?;
            buff.push_value(43)?;
            buff.push_value(32)?;
            Ok(())
        }()
        .expect("The overflow should not happend in this test case");

        assert_eq!(Some(32), buff.next_value());
        assert_eq!(Some(2), buff.next_value());
        assert_eq!(Some(43), buff.next_value());
        assert_eq!(Some(32), buff.next_value());
    }

    #[test]
    fn test_should_get_none_when_buffer_empty_without_pushing_values() {
        let mut buff = RingBuffer::<u8, 1024>::new()
            .expect("Allocation should be successful in this test case");
        assert_eq!(None, buff.next_value())
    }

    #[test]
    fn test_should_get_none_when_buffer_empty_after_pushing_and_getting_values() {
        let mut buff = RingBuffer::<u8, 1024>::new()
            .expect("Allocation should be successful in this test case");
        || -> Result<(), RingBufferError<_>> {
            buff.push_value(32)?;
            buff.push_value(2)?;
            buff.push_value(43)?;
            buff.push_value(31)?;
            Ok(())
        }()
        .expect("The overflow should not happend in this test case");

        (1..5).for_each(|_| {
            buff.next_value();
        });

        assert_eq!(None, buff.next_value())
    }

    #[test]
    fn test_should_get_overflow_error_with_latest_insert_value_when_buffer_push_more_values_than_capacity(
    ) {
        let mut buff =
            RingBuffer::<u8, 3>::new().expect("Allocation should be successful in this test case");
        let latest = 33;
        let result = || -> Result<(), RingBufferError<u8>> {
            buff.push_value(32)?;
            buff.push_value(2)?;
            buff.push_value(43)?;
            buff.push_value(latest)?;
            Ok(())
        }();
        assert_eq!(Err(RingBufferError::OverflowError(latest)), result);
    }

    #[test]
    fn test_should_push_without_error_after_consuming_few_values_from_full_buffer() {
        let mut buff =
            RingBuffer::<u8, 4>::new().expect("Allocation should be successful in this test case");
        let latest = 33;
        || -> Result<(), RingBufferError<u8>> {
            buff.push_value(32)?;
            buff.push_value(2)?;
            buff.push_value(43)?;
            buff.push_value(46)?;
            buff.push_value(latest)?;
            Ok(())
        }()
        .expect_err("The overflow error is acceptable in this case");

        buff.next_value();
        buff.next_value();

        let expected_value = 99;
        assert_eq!(Ok(()), buff.push_value(expected_value));

        buff.next_value();
        buff.next_value();

        assert_eq!(Some(expected_value), buff.next_value());
    }
}
