//! `core/templates/ring_buffer.h`: the power-of-two ring buffer.
//!
//! The engine masks the read/write cursors instead of wrapping them, keeps one
//! slot free (`space_left` is the capacity minus the number of readable items
//! minus one) and returns the number of items actually moved. `read`, `copy`,
//! `find`, `advance_read`, `decrease_write`, both `write` forms and `resize` are
//! ported operation for operation, including the resize path that relocates a
//! wrapped tail when the buffer grows.
//!
//! `read` returns `None` where the engine logs `ERR_FAIL_COND_V` and returns a
//! default-constructed element.

#[derive(Clone, Debug)]
pub struct RingBuffer<T> {
    data: Vec<T>,
    read_pos: usize,
    write_pos: usize,
    size_mask: usize,
}

impl<T: Clone + Default + PartialEq> RingBuffer<T> {
    /// `RingBuffer(p_power = 0)`; the capacity is `1 << power` and one slot is
    /// always kept free.
    pub fn new(power: u32) -> Self {
        Self {
            data: Vec::new(),
            read_pos: 0,
            write_pos: 0,
            size_mask: 0,
        }
        .resized(power)
    }

    fn resized(mut self, power: u32) -> Self {
        self.resize(power);
        self
    }

    /// `inc`: returns the current value and advances the cursor modulo capacity.
    fn inc(pos: &mut usize, amount: usize, mask: usize) -> usize {
        let ret = *pos;
        *pos = (*pos + amount) & mask;
        ret
    }

    /// `read()`.
    pub fn read(&mut self) -> Option<T> {
        if self.data_left() < 1 {
            return None;
        }
        let index = Self::inc(&mut self.read_pos, 1, self.size_mask);
        Some(self.data[index].clone())
    }

    /// `read(T *p_buf, int p_size, bool p_advance)`.
    pub fn read_into(&mut self, buffer: &mut [T], size: usize, advance: bool) -> usize {
        let left = self.data_left();
        let size = left.min(size);
        let mut pos = self.read_pos;
        let mut to_read = size;
        let mut destination = 0;
        while to_read != 0 {
            let end = (pos + to_read).min(self.size());
            let total = end - pos;
            for i in 0..total {
                if destination >= buffer.len() {
                    break;
                }
                buffer[destination] = self.data[pos + i].clone();
                destination += 1;
            }
            to_read -= total;
            pos = 0;
        }
        if advance {
            Self::inc(&mut self.read_pos, size, self.size_mask);
        }
        size
    }

    /// `copy(T *p_buf, int p_offset, int p_size)`.
    pub fn copy_into(&self, buffer: &mut [T], offset: usize, size: usize) -> usize {
        let left = self.data_left();
        let mut size = size;
        if offset + size > left {
            size = left.saturating_sub(offset);
            if size == 0 {
                return 0;
            }
        }
        size = left.min(size);
        let mut pos = self.read_pos;
        Self::inc(&mut pos, offset, self.size_mask);
        let mut to_read = size;
        let mut destination = 0;
        while to_read != 0 {
            let end = (pos + to_read).min(self.size());
            let total = end - pos;
            for i in 0..total {
                if destination >= buffer.len() {
                    break;
                }
                buffer[destination] = self.data[pos + i].clone();
                destination += 1;
            }
            to_read -= total;
            pos = 0;
        }
        size
    }

    /// `find(const T &t, int p_offset, int p_max_size)`: an index relative to
    /// `offset`, `0` when the range is empty and `-1` when the item is absent.
    pub fn find(&self, item: &T, offset: usize, max_size: usize) -> i32 {
        let left = self.data_left();
        let mut max_size = max_size;
        if offset + max_size > left {
            max_size = left.saturating_sub(offset);
            if max_size == 0 {
                return 0;
            }
        }
        max_size = left.min(max_size);
        let mut pos = self.read_pos;
        Self::inc(&mut pos, offset, self.size_mask);
        let mut to_read = max_size;
        while to_read != 0 {
            let end = (pos + to_read).min(self.size());
            let total = end - pos;
            for i in 0..total {
                if self.data[pos + i] == *item {
                    return (i + (max_size - to_read)) as i32;
                }
            }
            to_read -= total;
            pos = 0;
        }
        -1
    }

    /// `advance_read`.
    pub fn advance_read(&mut self, amount: usize) -> usize {
        let amount = amount.min(self.data_left());
        Self::inc(&mut self.read_pos, amount, self.size_mask);
        amount
    }

    /// `decrease_write`.
    pub fn decrease_write(&mut self, amount: usize) -> usize {
        let amount = amount.min(self.data_left());
        Self::inc(
            &mut self.write_pos,
            self.size_mask + 1 - amount,
            self.size_mask,
        );
        amount
    }

    /// `write(const T &)`: false when the buffer is full.
    pub fn write(&mut self, value: T) -> bool {
        if self.space_left() < 1 {
            return false;
        }
        let index = Self::inc(&mut self.write_pos, 1, self.size_mask);
        self.data[index] = value;
        true
    }

    /// `write(const T *, int)`: returns the number of items written.
    pub fn write_slice(&mut self, source: &[T]) -> usize {
        let left = self.space_left();
        let size = left.min(source.len());
        let mut pos = self.write_pos;
        let mut to_write = size;
        let mut src = 0;
        while to_write != 0 {
            let end = (pos + to_write).min(self.size());
            let total = end - pos;
            for i in 0..total {
                if src >= source.len() {
                    break;
                }
                self.data[pos + i] = source[src].clone();
                src += 1;
            }
            to_write -= total;
            pos = 0;
        }
        Self::inc(&mut self.write_pos, size, self.size_mask);
        size
    }

    /// `space_left`.
    pub fn space_left(&self) -> usize {
        if self.size() == 0 {
            return 0;
        }
        let left = self.read_pos as isize - self.write_pos as isize;
        if left < 0 {
            (self.size() as isize + left - 1) as usize
        } else if left == 0 {
            self.size() - 1
        } else {
            (left - 1) as usize
        }
    }

    /// `data_left`.
    pub fn data_left(&self) -> usize {
        self.size().saturating_sub(self.space_left() + 1)
    }

    /// `size`: the raw capacity (one slot is reserved).
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// The number of items that can still be written.
    pub fn capacity_left(&self) -> usize {
        self.space_left()
    }

    /// `clear`.
    pub fn clear(&mut self) {
        self.read_pos = 0;
        self.write_pos = 0;
    }

    /// `resize(int p_power)`.
    pub fn resize(&mut self, power: u32) {
        let old_size = self.size();
        let new_size = 1usize << power;
        let mask = new_size - 1;
        self.data.resize(new_size, T::default());
        if old_size < new_size && self.read_pos > self.write_pos {
            for i in 0..self.write_pos {
                self.data[(old_size + i) & mask] = self.data[i].clone();
            }
            self.write_pos = (old_size + self.write_pos) & mask;
        } else {
            self.read_pos &= mask;
            self.write_pos &= mask;
        }
        self.size_mask = mask;
    }
}

impl<T: Clone + Default + PartialEq> Default for RingBuffer<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_and_available_space() {
        let buffer: RingBuffer<u8> = RingBuffer::new(2);
        assert_eq!(buffer.size(), 4);
        assert_eq!(buffer.space_left(), 3, "one slot is always reserved");
        assert_eq!(buffer.data_left(), 0);
        assert_eq!(RingBuffer::<u8>::default().size(), 1);
    }

    #[test]
    fn single_writes_and_reads() {
        let mut buffer: RingBuffer<u8> = RingBuffer::new(2);
        assert!(buffer.write(1));
        assert!(buffer.write(2));
        assert!(buffer.write(3));
        assert!(!buffer.write(4), "full at capacity - 1");
        assert_eq!(buffer.data_left(), 3);
        assert_eq!(buffer.read(), Some(1));
        assert_eq!(buffer.read(), Some(2));
        assert_eq!(buffer.read(), Some(3));
        assert_eq!(buffer.read(), None);
    }

    #[test]
    fn wraparound_reads_and_copies() {
        let mut buffer: RingBuffer<u8> = RingBuffer::new(3);
        assert_eq!(buffer.write_slice(&[0, 1, 2, 3, 4, 5, 6]), 7);
        // Shift the cursor so the readable data wraps around the end.
        assert_eq!(buffer.read(), Some(0));
        assert_eq!(buffer.read(), Some(1));
        assert_eq!(buffer.read(), Some(2));
        assert_eq!(buffer.write_slice(&[7, 8, 9, 10]), 3);
        assert_eq!(buffer.data_left(), 7);

        let mut copied = [0u8; 7];
        assert_eq!(buffer.copy_into(&mut copied, 0, 7), 7);
        assert_eq!(copied, [3, 4, 5, 6, 7, 8, 9]);
        // copy does not consume.
        assert_eq!(buffer.data_left(), 7);

        // Offsets are relative to the read cursor.
        let mut tail = [0u8; 2];
        assert_eq!(buffer.copy_into(&mut tail, 5, 2), 2);
        assert_eq!(tail, [8, 9]);

        let mut all = [0u8; 7];
        assert_eq!(buffer.read_into(&mut all, 7, true), 7);
        assert_eq!(all, [3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(buffer.data_left(), 0);
    }

    #[test]
    fn reads_clamp_to_available_data() {
        let mut buffer: RingBuffer<u8> = RingBuffer::new(2);
        buffer.write_slice(&[1, 2]);
        let mut out = [0u8; 5];
        assert_eq!(buffer.read_into(&mut out, 5, false), 2);
        assert_eq!(buffer.data_left(), 2, "advance was false");
        assert_eq!(buffer.read_into(&mut out, 1, true), 1);
        assert_eq!(buffer.data_left(), 1);
    }

    #[test]
    fn find_with_offsets() {
        let mut buffer: RingBuffer<u8> = RingBuffer::new(3);
        buffer.write_slice(&[10, 20, 30, 20, 40]);
        assert_eq!(buffer.find(&20, 0, 5), 1);
        assert_eq!(buffer.find(&20, 2, 3), 1, "index relative to the offset");
        assert_eq!(buffer.find(&99, 0, 5), -1);
        // An offset past the data reports 0, like the engine.
        assert_eq!(buffer.find(&10, 9, 5), 0);
    }

    #[test]
    fn advance_and_decrease_write() {
        let mut buffer: RingBuffer<u8> = RingBuffer::new(2);
        buffer.write_slice(&[1, 2, 3]);
        assert_eq!(buffer.advance_read(2), 2);
        assert_eq!(buffer.data_left(), 1);
        assert_eq!(buffer.read(), Some(3));
        assert_eq!(buffer.advance_read(5), 0);
        assert!(buffer.write_slice(&[4, 5, 6]).eq(&3));

        // decrease_write gives back the last written items.
        assert_eq!(buffer.decrease_write(2), 2);
        assert_eq!(buffer.data_left(), 1);
    }

    #[test]
    fn resize_preserves_wrapped_data() {
        let mut buffer: RingBuffer<u8> = RingBuffer::new(2);
        buffer.write_slice(&[1, 2, 3]);
        assert_eq!(buffer.read(), Some(1));
        assert_eq!(buffer.read(), Some(2));
        // write_pos is behind read_pos: the data wraps.
        buffer.write_slice(&[4, 5]);
        let mut out = [0u8; 3];
        assert_eq!(buffer.copy_into(&mut out, 0, 3), 3);

        buffer.resize(4);
        assert_eq!(buffer.size(), 16);
        let mut out2 = [0u8; 3];
        assert_eq!(buffer.copy_into(&mut out2, 0, 3), 3);
        assert_eq!(out2, out, "growing relocates the wrapped tail");

        buffer.clear();
        assert_eq!(buffer.data_left(), 0);
        buffer.resize(2);
        assert_eq!(buffer.size(), 4);
    }

    #[test]
    fn shrink_keeps_cursors_in_range() {
        let mut buffer: RingBuffer<u8> = RingBuffer::new(4);
        buffer.write_slice(&[1, 2, 3, 4]);
        buffer.resize(2);
        assert_eq!(buffer.size(), 4);
        assert!(buffer.data_left() <= 3);
    }
}
