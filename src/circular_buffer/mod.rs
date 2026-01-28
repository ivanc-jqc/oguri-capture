use std::io::Error;

pub struct CircularBuffer<OGURI> {
    data: Vec<Option<OGURI>>,
    capacity: usize,
    head: usize,
    tail: usize,
    empty: bool,
}
impl<OGURI: Clone> CircularBuffer<OGURI> {
    fn new(capacity: usize) -> Self {
        Self { 
            data: vec![None; capacity],
            capacity: capacity, 
            head: 0,
            tail: 0,
            empty: true,
        }
    }

    fn push(element: OGURI) {

    }
    fn pop(&mut self) -> Option<&OGURI> {
        if self.empty { 
            return None;
        }
        let ble = self.data[0].as_ref().unwrap();
        Some(ble)
    }

}




//let example = CircularBuffer::new()
//example.push(blahblah)
//example.pop(blahlblah)
