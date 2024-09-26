use std::collections::{HashSet, VecDeque};
use std::env;
use std::str::from_utf8;

use jack_ast::{gramar::*, tokens::JackSymbol};
use vm_parser::*;

use crate::{
    class::JackClassCompilerContext,
    vars::{JackTableNames, JackVariable},
};

const ADD: AsmInstructionPayload = AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Add);

const POP_THAT: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::Pointer,
    kind: AsmMemoryInstructionKind::Pop,
    val: 1,
});

const POP_TEMP: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::Temp,
    kind: AsmMemoryInstructionKind::Pop,
    val: 0,
});

const PUSH_TEMP: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::Temp,
    kind: AsmMemoryInstructionKind::Push,
    val: 0,
});

const PUSH_THIS: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::Pointer,
    kind: AsmMemoryInstructionKind::Push,
    val: 0,
});

const PUSH_ARG_0: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::Arg,
    kind: AsmMemoryInstructionKind::Push,
    val: 0,
});

const POP_THIS: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::Pointer,
    kind: AsmMemoryInstructionKind::Pop,
    val: 0,
});

const PUSH_THAT_0: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::That,
    kind: AsmMemoryInstructionKind::Push,
    val: 0,
});

const POP_THAT_0: AsmInstructionPayload = AsmInstructionPayload::Memory(AsmMemoryInstruction {
    segment: AsmMemoryInstructionSegment::That,
    kind: AsmMemoryInstructionKind::Pop,
    val: 0,
});

const DROP_STACK_VALUE: AsmInstructionPayload =
    AsmInstructionPayload::Memory(AsmMemoryInstruction {
        segment: AsmMemoryInstructionSegment::Temp,
        kind: AsmMemoryInstructionKind::Pop,
        val: 0,
    });

pub struct JackSubroutineCompilerContext<'a> {
    class: &'a JackClassCompilerContext,
    subroutine: &'a mut JackSubroutine,
    skip_vars_check: bool,
    statement_idx: usize,
    if_counter: usize,
    while_counter: usize,
    acc: VecDeque<AsmInstructionPayload>,
    vars: JackTableNames,
    assignments: HashSet<&'a JackVariableName>,
}

impl<'a> JackSubroutineCompilerContext<'a> {
    pub fn init(
        class: &'a JackClassCompilerContext,
        subroutine: &'a mut JackSubroutine,
        skip_vars_check: bool,
    ) -> Self {
        let mut global = JackTableNames::default();

        if subroutine.key == JackSubroutineType::Method {
            let kind = JackType::Class(class.class().0.clone());
            let name = JackVariableName(b"__CLASS_NAME__".to_vec());
            let mut declaration = JackDeclaration {
                segment: JackSegment::Arg,
                kind,
                names: vec![name],
            };
            global.migrate(&mut declaration)
        }

        for i in subroutine.vars.iter_mut() {
            global.migrate(i)
        }

        let mut res = Self {
            class,
            subroutine,
            vars: global,
            statement_idx: 0,
            if_counter: 0,
            while_counter: 0,
            acc: VecDeque::new(),
            assignments: HashSet::new(),
            skip_vars_check,
        };

        res.compile_function_header();

        res
    }

    fn compile_function_header(&mut self) {
        let mut full_function_name = self.class.class().0.to_vec();
        full_function_name.push(b'.');

        full_function_name.extend(&self.subroutine.name.0);

        let asm =
            AsmInstructionPayload::Function(AsmFunctionInstruction::Definition(FunctionMetadata {
                name: full_function_name,
                args_count: self.vars.local() as i16,
            }));

        match self.subroutine.key {
            JackSubroutineType::Method => {
                self.acc.push_back(asm);
                self.acc.push_back(PUSH_ARG_0);
                self.acc.push_back(POP_THIS);
            }
            JackSubroutineType::Function => self.acc.push_back(asm),
            JackSubroutineType::Constructor => {
                self.acc.push_back(asm);
                self.push_const(self.class.vars.fields() as i16);
                self.acc.push_back({
                    AsmInstructionPayload::Function(AsmFunctionInstruction::Call(
                        FunctionMetadata {
                            name: b"Memory.alloc".to_vec(),
                            args_count: 1,
                        },
                    ))
                });
                self.acc.push_back(POP_THIS);
            }
        }
    }

    fn is_assigned(&self, name: &JackVariableName) -> bool {
        self.assignments.contains(name)
    }

    fn get(&self, name: &JackVariableName, skip_checks: bool) -> Option<&JackVariable> {
        if let Some(var) = self.vars.get(name) {
            if var.segment != JackSegment::Arg
                && !self.is_assigned(name)
                && !skip_checks
                && !self.skip_vars_check
            {
                panic!(
                    "Try use variable {} before assign it",
                    from_utf8(&name.0).unwrap()
                );
            }
            return Some(var);
        }

        if let Some(var) = self.class.vars.get(name) {
            if self.subroutine.key == JackSubroutineType::Function && var.segment == JackSegment::Field {
                return None;
            }

            if var.segment != JackSegment::Static
                && self.subroutine.key == JackSubroutineType::Constructor
                && !self.is_assigned(name)
                && !skip_checks
                && !self.skip_vars_check
            {
                panic!(
                    "Try use variable {} before assign it",
                    from_utf8(&name.0).unwrap()
                );
            }
            return Some(var);
        }

        None
    }

    fn assign(&mut self, name: &'a JackVariableName) -> bool {
        self.assignments.insert(name)
    }

    fn push_variable(&mut self, ident: &JackVariableName) {
        let is_strict = if let Ok(v) = env::var("STRICT_MODE") {
            v == "1"
        } else {
            false
        };
        match self.get(&ident, false) {
            None => {
                if is_strict {
                    panic!("Variable {} not declared", from_utf8(&ident.0).unwrap())
                }
                self.push_const(0);
            }
            Some(v) => {
                self.acc.push_back(v.as_asm());
            }
        };
    }

    fn compile_statement(&mut self, statement: &'a mut JackStatement) {
        match statement {
            JackStatement::Do(do_statement) => {
                self.compile_term(&mut do_statement.call);
                self.acc.push_back(DROP_STACK_VALUE);
            }
            JackStatement::Return(return_statement) => {
                match &mut return_statement.expression {
                    Some(e) => {
                        if self.subroutine.kind.is_void() {
                            panic!("Void method should return nothing")
                        }

                        if !e.is_this() && self.subroutine.key == JackSubroutineType::Constructor {
                            panic!("Constructor should return this")
                        }

                        self.compile_expression(e);
                    }
                    None => {
                        if !self.subroutine.kind.is_void() {
                            panic!("Non void method should a value")
                        }
                        self.push_const(0);
                    }
                }
                self.acc.push_back(AsmInstructionPayload::Function(
                    AsmFunctionInstruction::Return,
                ));
            }
            JackStatement::Let(let_statement) => match &mut let_statement.variable.payload {
                JackTermPayload::ArrayElem(ident, expr) => {
                    if let Some(var) = self.get(ident, false) {
                        self.acc.push_back(var.as_asm());
                        self.compile_expression(expr.as_mut());
                        self.acc.push_back(ADD);
                        self.compile_expression(&mut let_statement.expression);
                        self.acc.push_back(POP_TEMP);
                        self.acc.push_back(POP_THAT);
                        self.acc.push_back(PUSH_TEMP);
                        self.acc.push_back(POP_THAT_0);
                    } else {
                        panic!("Not found variable {}", from_utf8(&ident.0).unwrap())
                    };
                }
                JackTermPayload::Ident(ident) => {
                    self.compile_expression(&mut let_statement.expression);
                    if let Some(var) = self.get(ident, true) {
                        self.acc.push_back(var.as_assign_asm());
                        self.assign(ident)
                    } else {
                        panic!("Not found variable {}", from_utf8(&ident.0).unwrap())
                    };
                }
                _ => unreachable!(),
            },
            JackStatement::If(if_statement) => {
                self.compile_term(&mut if_statement.condition);

                let [true_val, false_val, end_val] = self.build_if_names();

                self.acc
                    .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                        kind: AsmBranchInstructionKind::IfGoto,
                        name: true_val.clone(),
                    }));

                self.acc
                    .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                        kind: AsmBranchInstructionKind::Goto,
                        name: false_val.clone(),
                    }));

                self.acc
                    .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                        kind: AsmBranchInstructionKind::Label,
                        name: true_val,
                    }));

                for statement in if_statement.statements.0.iter_mut() {
                    self.compile_statement(statement);
                }

                let false_label = AsmInstructionPayload::Branch(AsmBranchInstruction {
                    kind: AsmBranchInstructionKind::Label,
                    name: false_val,
                });

                if let Some(statements) = if_statement.else_statements.as_mut() {
                    self.acc
                        .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                            kind: AsmBranchInstructionKind::Goto,
                            name: end_val.clone(),
                        }));
                    self.acc.push_back(false_label);
                    for statement in statements.0.iter_mut() {
                        self.compile_statement(statement);
                    }
                    self.acc
                        .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                            kind: AsmBranchInstructionKind::Label,
                            name: end_val,
                        }));
                } else {
                    self.acc.push_back(false_label);
                }
            }
            JackStatement::While(while_statement) => {
                let [exp_val, end_val] = self.build_while_names();

                self.acc
                    .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                        kind: AsmBranchInstructionKind::Label,
                        name: exp_val.clone(),
                    }));

                self.compile_term(&mut while_statement.condition);
                self.acc.push_back(AsmInstructionPayload::Arithmetic(
                    AsmArithmeticInstruction::Not,
                ));

                self.acc
                    .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                        kind: AsmBranchInstructionKind::IfGoto,
                        name: end_val.clone(),
                    }));

                for statement in while_statement.statements.0.iter_mut() {
                    self.compile_statement(statement);
                }

                self.acc
                    .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                        kind: AsmBranchInstructionKind::Goto,
                        name: exp_val,
                    }));

                self.acc
                    .push_back(AsmInstructionPayload::Branch(AsmBranchInstruction {
                        kind: AsmBranchInstructionKind::Label,
                        name: end_val,
                    }));
            }
        }
    }

    fn build_if_names(&mut self) -> [Vec<u8>; 3] {
        let mut res = [b"IF_".to_vec(), b"IF_".to_vec(), b"IF_".to_vec()];

        res[0].extend(b"TRUE_");
        res[1].extend(b"FALSE_");
        res[2].extend(b"END_");

        for i in res.iter_mut() {
            i.extend(&self.class.class().0);
            i.push(b'_');
            i.extend(&self.subroutine.name.0);
            i.push(b'_');
            i.extend(self.if_counter.to_string().as_bytes())
        }

        self.if_counter += 1;

        res
    }

    fn build_while_names(&mut self) -> [Vec<u8>; 2] {
        let mut res = [b"WHILE_".to_vec(), b"WHILE_".to_vec()];

        res[0].extend(b"EXP_");
        res[1].extend(b"END_");

        for i in res.iter_mut() {
            i.extend(&self.class.class().0);
            i.push(b'_');
            i.extend(&self.subroutine.name.0);
            i.push(b'_');
            i.extend(self.while_counter.to_string().as_bytes())
        }

        self.while_counter += 1;

        res
    }

    fn compile_term(&mut self, term: &mut JackTerm) {
        match &mut term.payload {
            JackTermPayload::Int(integer) => self.push_const(integer.to_int()),
            JackTermPayload::String(string) => {
                let size = string.0.len();
                self.push_const(size as i16);
                self.acc.push_back(AsmInstructionPayload::Function(
                    AsmFunctionInstruction::Call(FunctionMetadata {
                        name: b"String.new".to_vec(),
                        args_count: 1,
                    }),
                ));

                for i in string.0.iter() {
                    self.push_const(*i as i16);
                    self.acc.push_back(AsmInstructionPayload::Function(
                        AsmFunctionInstruction::Call(FunctionMetadata {
                            name: b"String.appendChar".to_vec(),
                            args_count: 2,
                        }),
                    ));
                }
            }
            JackTermPayload::Const(JackConstantTerm::This) => self.acc.push_back(PUSH_THIS),
            JackTermPayload::Const(JackConstantTerm::True) => {
                self.push_const(1);
                self.acc.push_back(AsmInstructionPayload::Arithmetic(
                    AsmArithmeticInstruction::Neg,
                ));
            }
            JackTermPayload::Const(_) => self.push_const(0),
            JackTermPayload::Expression(expr) => {
                self.compile_expression(expr.as_mut());
            }
            JackTermPayload::Ident(ident) => {
                self.push_variable(ident);
            }
            JackTermPayload::Unary(op, t) => {
                self.compile_term(t.as_mut());
                self.acc.push_back(match op {
                    JackSymbol::Not => {
                        AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Not)
                    }
                    JackSymbol::Minus => {
                        AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Neg)
                    }
                    _ => unreachable!(),
                });
            }
            JackTermPayload::MethodCall(method_name, expressions) => {
                self.acc.push_back(PUSH_THIS);
                self.call_function(&self.class.class().0, method_name, expressions, true);
            }

            JackTermPayload::FunctionCall(class_name, method_name, expressions) => {
                match self.get(class_name, false) {
                    None => self.call_function(&class_name.0, method_name, expressions, false),
                    Some(var1) => {
                        let v = var1 as *const JackVariable;
                        let var = unsafe { &*v };
                        self.acc.push_back(var.as_asm());
                        self.call_function(var.kind.as_slice(), method_name, expressions, true)
                    }
                }
            }
            JackTermPayload::ArrayElem(ident, expr) => match self.get(ident, false) {
                None => panic!("Not found array {}", from_utf8(&ident.0).unwrap()),
                Some(v) if v.kind.is_array() => {
                    self.acc.push_back(v.as_asm());
                    self.compile_expression(expr.as_mut());
                    self.acc.push_back(ADD);
                    self.acc.push_back(POP_THAT);
                    self.acc.push_back(PUSH_THAT_0);
                }
                _ => panic!("{} is not an array", from_utf8(&ident.0).unwrap()),
            },
        }
    }

    fn call_function(
        &mut self,
        class_name: &[u8],
        function_name: &JackVariableName,
        expressions: &mut JackExpressions,
        is_method: bool,
    ) {
        let mut args_count = is_method as i16;
        for expression in expressions.data.iter_mut() {
            args_count += 1;
            self.compile_expression(expression);
        }

        let mut full_function_name = class_name.to_vec();
        full_function_name.push(b'.');

        full_function_name.extend(&function_name.0);

        self.acc.push_back(AsmInstructionPayload::Function(
            AsmFunctionInstruction::Call(FunctionMetadata {
                name: full_function_name,
                args_count,
            }),
        ));
    }

    fn compile_expression(&mut self, expression: &mut JackExpression) {
        self.compile_term(&mut expression.term);

        for (op, term) in expression.extra.iter_mut() {
            self.compile_term(term);
            self.compile_op(op);
        }
    }

    fn push_const(&mut self, i: i16) {
        self.acc
            .push_back(AsmInstructionPayload::Memory(AsmMemoryInstruction {
                segment: AsmMemoryInstructionSegment::Const,
                kind: AsmMemoryInstructionKind::Push,
                val: i,
            }))
    }

    fn compile_op(&mut self, op: &JackSymbol) {
        let asm_command = match op {
            JackSymbol::Plus => ADD,
            JackSymbol::Minus => AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Sub),
            JackSymbol::And => AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::And),
            JackSymbol::Or => AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Or),
            JackSymbol::Eq => AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Eq),
            JackSymbol::Greater => AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Gt),
            JackSymbol::Less => AsmInstructionPayload::Arithmetic(AsmArithmeticInstruction::Lt),
            JackSymbol::Multiply => {
                AsmInstructionPayload::Function(AsmFunctionInstruction::Call(FunctionMetadata {
                    name: b"Math.multiply".to_vec(),
                    args_count: 2,
                }))
            }
            JackSymbol::Divide => {
                AsmInstructionPayload::Function(AsmFunctionInstruction::Call(FunctionMetadata {
                    name: b"Math.divide".to_vec(),
                    args_count: 2,
                }))
            }
            _ => panic!("unknown op"),
        };

        self.acc.push_back(asm_command)
    }
}

impl Iterator for JackSubroutineCompilerContext<'_> {
    type Item = AsmInstructionPayload;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.acc.pop_front() {
            return Some(v);
        }

        while self.statement_idx < self.subroutine.statements.0.len() {
            let link =
                self.subroutine.statements.0[self.statement_idx].as_mut() as *mut JackStatement;
            self.compile_statement(unsafe { &mut *link });
            self.statement_idx += 1;
            if let Some(v) = self.acc.pop_front() {
                return Some(v);
            }
        }

        None
    }
}
