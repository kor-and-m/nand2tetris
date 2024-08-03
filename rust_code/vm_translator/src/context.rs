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

    pub fn set_new_pointer(&mut self, pointer: usize) {
        self.pointer += pointer;
    }

    pub fn global_instruction_number(&mut self, instruction_idx: usize) -> usize {
        self.pointer / 17 + instruction_idx
    }

    pub fn set_intruction(&mut self, instruction: Vec<u8>, instruction_idx: usize) {
        self.pointer_map
            .entry(instruction)
            .or_insert(Vec::new())
            .push(self.pointer + instruction_idx * 17)
    }
}
