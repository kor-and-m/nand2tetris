use std::collections::HashMap;

#[derive(Debug)]
pub struct WriteFileContext {
    pointer: usize,
    pub pointer_map: HashMap<Vec<u8>, Vec<usize>>,
}

impl WriteFileContext {
    pub fn new() -> Self {
        Self {
            pointer: 0,
            pointer_map: HashMap::new(),
        }
    }

    pub fn global_instruction_number(&self) -> usize {
        self.pointer
    }

    pub fn incr(&mut self) {
        self.pointer += 1;
    }

    pub fn set_intruction(&mut self, instruction: Vec<u8>) {
        self.pointer_map
            .entry(instruction)
            .or_insert(Vec::new())
            .push(self.pointer)
    }
}
