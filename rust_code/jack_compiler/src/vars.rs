use std::collections::HashMap;

use jack_ast::gramar::*;
use vm_parser::{
    AsmInstructionPayload, AsmMemoryInstruction, AsmMemoryInstructionKind,
    AsmMemoryInstructionSegment,
};

#[derive(Debug)]
pub struct JackVariable {
    pub kind: JackType,
    pub segment: JackSegment,
    pub idx: u8,
}

impl JackVariable {
    pub fn new(kind: JackType, segment: JackSegment, idx: u8) -> Self {
        Self { kind, segment, idx }
    }

    pub fn as_asm(&self) -> AsmInstructionPayload {
        let segment = match self.segment {
            JackSegment::Arg => AsmMemoryInstructionSegment::Arg,
            JackSegment::Lcl => AsmMemoryInstructionSegment::Local,
            JackSegment::Field => AsmMemoryInstructionSegment::This,
            JackSegment::Static => AsmMemoryInstructionSegment::Static,
        };

        AsmInstructionPayload::Memory(AsmMemoryInstruction {
            segment,
            kind: AsmMemoryInstructionKind::Push,
            val: self.idx as i16,
        })
    }

    pub fn as_assign_asm(&self) -> AsmInstructionPayload {
        let segment = match self.segment {
            JackSegment::Arg => AsmMemoryInstructionSegment::Arg,
            JackSegment::Lcl => AsmMemoryInstructionSegment::Local,
            JackSegment::Field => AsmMemoryInstructionSegment::This,
            JackSegment::Static => AsmMemoryInstructionSegment::Static,
        };

        AsmInstructionPayload::Memory(AsmMemoryInstruction {
            segment,
            kind: AsmMemoryInstructionKind::Pop,
            val: self.idx as i16,
        })
    }
}

#[derive(Default)]
struct JackTableNamesCounter {
    arg: u8,
    lcl: u8,
    global: u8,
    field: u8,
}

impl JackTableNamesCounter {
    pub fn get(&mut self, segment: &JackSegment) -> u8 {
        let counter = match segment {
            JackSegment::Arg => &mut self.arg,
            JackSegment::Lcl => &mut self.lcl,
            JackSegment::Field => &mut self.field,
            JackSegment::Static => &mut self.global,
        };

        *counter += 1;
        *counter - 1
    }
}

#[derive(Default)]
pub struct JackTableNames {
    map: HashMap<JackVariableName, JackVariable>,
    counter: JackTableNamesCounter,
}

impl JackTableNames {
    pub fn migrate(&mut self, declaration: &mut JackDeclaration) {
        let kind = declaration.kind.take();
        let segment = declaration.segment;
        let first_variable = self.new_variable(kind, segment);

        let mut names_iter = declaration.names.iter_mut();
        let first_nmae = names_iter.next().unwrap().take();

        for i in names_iter {
            let v = self.new_next_variable(&first_variable);
            self.map.insert(i.take(), v);
        }

        self.map.insert(first_nmae, first_variable);
    }

    pub fn get(&self, name: &JackVariableName) -> Option<&JackVariable> {
        self.map.get(name)
    }

    pub fn local(&self) -> u8 {
        self.counter.lcl
    }

    pub fn fields(&self) -> u8 {
        self.counter.field
    }

    fn new_variable(&mut self, kind: JackType, segment: JackSegment) -> JackVariable {
        let idx = self.counter.get(&segment);
        JackVariable::new(kind, segment, idx)
    }

    fn new_next_variable(&mut self, variable: &JackVariable) -> JackVariable {
        self.new_variable(variable.kind.clone(), variable.segment)
    }
}
