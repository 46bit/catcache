use std::collections::VecDeque;

pub struct FIFOBuffer<T>
    where T: Sync,
          T: Send
{
    items: VecDeque<T>,
    desired_buffering: usize,
}

impl<T> FIFOBuffer<T>
    where T: Sync,
          T: Send
{
    pub fn new(desired_buffering: usize) -> FIFOBuffer<T> {
        FIFOBuffer::<T> {
            items: VecDeque::with_capacity(desired_buffering),
            desired_buffering: desired_buffering,
        }
    }

    pub fn shift(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn push(&mut self, item: T) {
        self.items.push_back(item);
    }

    pub fn topup(&mut self) -> Option<usize> {
        let items_len = self.items.len();
        if items_len < self.desired_buffering {
            return Some(self.desired_buffering - items_len);
        }
        None
    }
}
