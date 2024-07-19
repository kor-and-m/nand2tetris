mod arithmetic;
mod constants;
mod memory;
mod model;

pub use model::Translator;

// fn write_token<'a, T>(
//     token: &'a Token,
//     factory: &'a mut VariableFactory<'a>,
//     f: T
// ) -> () where T: FnMut(Instruction<'a>) -> bool {

// }

// pub fn write_token_to_buff(
//     buff: &mut [u8],
//     token: &Token,
//     factory: &mut VariableFactory,
// ) -> Result<usize, &'static str> {
//     match &token.payload {
//         TokenPayload::Memory(memory) => memory::write_memory_token_to_buff(buff, memory, factory),
//         TokenPayload::Arithmetic(arithmetic) => {
//             arithmetic::write_arithmetic_token_to_buff(buff, arithmetic, factory)
//         }
//     }
// }
