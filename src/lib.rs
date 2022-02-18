#[cfg(test)]
pub mod ring_buffer;
mod tests {
    use crate::ring_buffer::{RingBuffer, RingBufferError};
    
    #[test]
    fn it_works() {
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
}
