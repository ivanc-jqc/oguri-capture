use std::{cell::{Cell, UnsafeCell}, mem::MaybeUninit, ops::{Deref, DerefMut}};

// pub struct RingBuffer<T, const N: usize> {
//     buffer: [UnsafeCell<MaybeUninit<T>>; N],
//     head: usize
// }
// impl<T, const N: usize> RingBuffer<T, N> {
//     pub fn new() -> Self {  
//         const temp = UnsafeCell::<MaybeUninit<T>>::new(MaybeUninit::uninit());
//         Self { buffer: [;N ], head: () }
//     }
// }








pub struct CircularBuffer<OGURI> {
    data: Vec<Option<OGURI>>,
    capacity: usize,
    head: usize,
    wrapped: bool,
}
impl<OGURI: Clone + std::fmt::Debug> CircularBuffer<OGURI> {
    pub fn new(capacity: usize) -> Self {
        Self { 
            data: vec![None; capacity],
            capacity: capacity, 
            head: 0,
            wrapped: false,
        }
    }

    pub fn push(&mut self, element: OGURI) {
        let temp = element.clone();
        self.data[self.head] = Some(element);
        println!("head: {:?}", self.head);
        if self.head+1 == self.capacity {
            self.head = 0;
            self.wrapped = true;
        } 
        else {
            self.head += 1;
        }
        
    }
    fn rotate(&mut self) { 
        self.data.rotate_left(self.head);
    }
    pub fn dump(&mut self) -> &Vec<Option<OGURI>> {
        if (self.wrapped) {
            self.rotate();
        }
        &self.data
    }
    // for now.. not needed.
    // fn pop(&self) -> Option<&OGURI> {
    //     if self.head == self.tail{ 
    //         return None;
    //     }
    //     let ble = self.data[0].as_ref().unwrap();
    //     Some(ble)
    // }

}

impl<OGURI> Deref for CircularBuffer<OGURI> {
    type Target = Vec<Option<OGURI>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<OGURI> DerefMut for CircularBuffer<OGURI> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        let mut test: CircularBuffer<f32> = CircularBuffer::new(5.try_into().unwrap());
        dbg!(test.data);
        // test.push(5f32);
        // let result = test.pop().unwrap();
        // assert_eq!(5f32, *result);
    }

    #[test]
    fn simple_push_pop() {
        let mut test: CircularBuffer<f32> = CircularBuffer::new(5.try_into().unwrap());
        test.push(1f32);
        test.push(2f32);
        test.push(3f32);
        test.push(4f32);
        test.push(5f32);
        test.push(6f32);
        test.push(7f32);
        dbg!(&test.data);
        test.rotate();
        dbg!(&test.data);
        // let result = test.pop().unwrap();
        // assert_eq!(5f32, *result);
    }
}


//let example = CircularBuffer::new()
//example.push(blahblah)
//example.pop(blahlblah)
