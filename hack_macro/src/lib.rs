use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DeriveInput, Expr, ExprAssign, ExprLit, Lit,
    Variant,
};

#[derive(Clone)]
struct ParsedEnumField {
    ident: String,
    lit: String,
    lit_len: usize,
}

impl ParsedEnumField {
    fn new(ident: String, lit: String, lit_len: usize) -> Self {
        Self {
            ident,
            lit,
            lit_len,
        }
    }

    fn to_write_symbols_format(&self) -> String {
        if self.lit_len == 0 {
            format!("Self::{i} => 0", i = self.ident)
        } else {
            format!(
                "Self::{i} => {{ buff[..{l}].copy_from_slice(b\"{v}\");\n {l} }}",
                i = self.ident,
                l = self.lit_len,
                v = self.lit
            )
        }
    }

    fn to_static_symbol_format(&self) -> String {
        format!("Self::{i} => b\"{v}\"", i = self.ident, v = self.lit)
    }

    fn from_static_symbol_format(&self) -> String {
        format!("b\"{v}\" => Self::{i}", i = self.ident, v = self.lit)
    }

    fn as_bytes_binary_format(&self) -> String {
        format!("Self::{i} => b\"{v}\"", i = self.ident, v = self.lit)
    }
}

fn parse_hack_attrs<'a>(attrs: &'a Vec<Attribute>) -> impl Iterator<Item = &'a Attribute> {
    attrs.iter().filter(|a| a.path().is_ident("hack"))
}

fn extract_binary_values<'a>(
    iter: impl Iterator<Item = &'a Attribute>,
    keys: &'a mut [(&'a str, String)],
) -> Vec<String> {
    let l = keys.len();
    let mut answers = Vec::with_capacity(keys.len());

    for i in 0..l {
        let v = &mut keys[i].1;
        answers.push(std::mem::take(v))
    }

    for attr in iter {
        let s = format!("{:?}", attr);
        let assign: ExprAssign = attr.parse_args().expect(&s);
        let key = if let Expr::Path(p) = *assign.left {
            p.path
                .get_ident()
                .expect("Can't get ident")
                .span()
                .source_text()
                .expect("Can't fetch source text from path")
        } else {
            panic!("left part should be symbol")
        };

        if let Some((idx, _)) = keys.iter().enumerate().find(|k| k.1 .0 == &key) {
            let v = extract_binary_value_from_expr(assign.right);
            answers[idx] = std::str::from_utf8(&v)
                .expect("Error parsing to utf8")
                .to_string();
        }
    }

    answers
}

fn extract_binary_value_from_expr(right: Box<Expr>) -> Vec<u8> {
    if let Expr::Lit(ExprLit {
        lit: Lit::ByteStr(l),
        ..
    }) = *right
    {
        l.value()
    } else {
        panic!("right side should be literal")
    }
}

fn parse_a_instraction(s: &str) -> String {
    if let Ok(v) = s.parse::<i32>() {
        format!("Instruction::A(AInstruction::Number({}))", v)
    } else {
        format!("Instruction::A(AInstruction::Const(AConst::{}))", s)
    }
}

fn parse_c_instraction(s: &str) -> String {
    let mut equal_idx: usize = 0;
    let mut semicolon: usize = s.len();

    let c = s.chars();

    for (i, v) in c.enumerate() {
        if v == '=' {
            equal_idx = i + 1;
        }

        if v == ';' {
            semicolon = i;
            break;
        }
    }

    format!(
        "
            Instruction::C(CInstruction{{
                dest: CInstructionDest::__from_static_symbols(b\"{dest_arg}\"),
                expression: CInstructionExpression::__from_static_symbols(b\"{expression_arg}\"),
                jump: CInstructionJump::__from_static_symbols(b\"{jump_arg}\"),
            }})
        ",
        dest_arg = &s[..equal_idx],
        expression_arg = &s[equal_idx..semicolon],
        jump_arg = &s[semicolon..]
    )
}

fn parse_helper_instraction(s: &str) -> String {
    let mut c = s.chars();

    match c.next() {
        Some('(') => {
            if let Some(')') = c.last() {
                let v: Vec<&str> = s[1..s.len() - 1].split("_").collect();

                if v.len() != 3 {
                    panic!("Label should be formated NAMESPACE_NAME_IDX")
                }

                if let Ok(label_number) = v[2].parse::<i32>() {
                    format!(
                        "Instruction::Helper(HelperInstruction::Label(LabelInstruction {{
                            prefix: b\"{prefix}\",
                            name: b\"{name}\",
                            prefix_len: {prefix_len},
                            name_len: {name_len},
                            idx: {label_number},
                        }}))",
                        label_number = label_number,
                        prefix = v[0],
                        name = v[1],
                        prefix_len = v[0].len(),
                        name_len = v[1].len()
                    )
                } else {
                    panic!("Wrong helper command")
                }
            } else {
                panic!("Wrong helper command")
            }
        }
        Some('/') => {
            if let Some('/') = c.next() {
                let start_comment = if let Some(' ') = c.next() { 3 } else { 2 };
                format!(
                    "Instruction::Helper(HelperInstruction::Comment(b\"{}\"))",
                    &s[start_comment..]
                )
            } else {
                panic!("Wrong helper command")
            }
        }
        _ => panic!("Wrong helper command"),
    }
}

#[proc_macro]
pub fn instruction(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expr = parse_macro_input!(input as Expr);
    let instruction: TokenStream = match expr {
        Expr::Lit(ExprLit {
            lit: Lit::ByteStr(l),
            ..
        }) => {
            let l_val = l.value();
            match l_val[0] {
                b'@' => {
                    let instruction_string = std::str::from_utf8(&l_val[1..])
                        .expect("Error parsing a A instruction to utf8");
                    parse_a_instraction(instruction_string)
                        .parse()
                        .expect("Error parsing a A instruction")
                }
                x if x == b'(' || x == b'/' => {
                    let instruction_string = std::str::from_utf8(&l_val)
                        .expect("Error parsing a helper instructions to utf8");
                    parse_helper_instraction(instruction_string)
                        .parse()
                        .expect("Error parsing a helper instructions")
                }
                _ => {
                    let instruction_string = std::str::from_utf8(&l_val)
                        .expect("Error parsing a C instructions to utf8");
                    parse_c_instraction(instruction_string)
                        .parse()
                        .expect("Error parsing a C instruction")
                }
            }
        }
        _ => panic!("Wrong instruction"),
    };
    TokenStream::from(instruction).into()
}

#[proc_macro_derive(BinaryInstruction, attributes(hack))]
pub fn declar_binary_methods(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident,
        data,
        attrs: _main_attrs,
        ..
    } = parse_macro_input!(input as DeriveInput);

    // let struct_type = extract_binary_values(
    //     parse_hack_attrs(&main_attrs),
    //     &mut [("union", String::new())],
    // );

    let enums = match data {
        Data::Enum(data_enum) => parse_binary_enums(data_enum),
        _ => panic!("Can't bederived for non enums"),
    };

    let as_bytes_body =
        parsed_enums_to_token_stream(&enums, ParsedEnumField::as_bytes_binary_format, true);

    let as_bytes_body_size: TokenStream = enums[0]
        .lit_len
        .to_string()
        .parse()
        .expect("Error parsing as_bytes_body");

    let expanded = quote! {
        impl #ident {
            pub const fn as_bytes_const(&self) -> &'static [u8; #as_bytes_body_size] {
                match self { #as_bytes_body }
            }
        }
    };
    TokenStream::from(expanded).into()
}

#[proc_macro_derive(SymbolicElem, attributes(hack))]
pub fn declar_as_a_constant(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident,
        data,
        attrs: main_attrs,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let enums = match data {
        Data::Enum(data_enum) => enum_data_to_tokens(data_enum, main_attrs),
        _ => panic!("Can't bederived for non enums"),
    };

    let write =
        parsed_enums_to_token_stream(&enums, ParsedEnumField::to_write_symbols_format, true);
    let from_static =
        parsed_enums_to_token_stream(&enums, ParsedEnumField::from_static_symbol_format, false);
    let to_static =
        parsed_enums_to_token_stream(&enums, ParsedEnumField::to_static_symbol_format, true);

    let expanded = quote! {
        impl #ident {
            pub const fn __from_static_symbols(symbols: &'static [u8]) -> Self {
                match symbols { #from_static }
            }

            pub const fn __as_static_symbols(&self) -> &'static [u8] {
                match self { #to_static }
            }
        }

        impl<'a> SymbolicElem<'a> for #ident {
            fn write_symbols(&self, buff: &mut [u8]) -> usize {
                match self { #write }
            }
        }
    };
    TokenStream::from(expanded).into()
}

fn parsed_enums_to_token_stream(
    enums: &Vec<ParsedEnumField>,
    f: fn(&ParsedEnumField) -> String,
    is_fully_matched: bool,
) -> TokenStream {
    let mut v: Vec<String> = enums.iter().map(f).collect();
    if !is_fully_matched {
        v.push("_ => unreachable!()".to_owned())
    }

    v.join(",\n").parse().expect("Error parsing stream")
}

fn parse_binary_enums(data: DataEnum) -> Vec<ParsedEnumField> {
    let variants_iter = data.variants.iter().map(parse_binary_elem);
    sort_parsed_enums(variants_iter.collect())
}

fn sort_parsed_enums(mut data: Vec<ParsedEnumField>) -> Vec<ParsedEnumField> {
    data.sort_by(|a, b| b.lit_len.cmp(&a.lit_len));
    data
}

fn parse_binary_elem(x: &Variant) -> ParsedEnumField {
    let mut variable_names = extract_binary_values(
        parse_hack_attrs(&x.attrs),
        &mut [("binary", "".to_owned()), ("int", "".to_owned())],
    );

    let variable_name = if let Ok(v) = variable_names[1].parse::<i16>() {
        format!("{:016b}", v)
    } else {
        if variable_names[0].len() > 0 {
            for i in variable_names[0].chars() {
                if i != '0' && i != '1' {
                    panic!("Instruction attribute binary should be made up from 0 and 1")
                }
            }
        } else {
            panic!("Instruction attributes weren't set")
        }

        std::mem::take(&mut variable_names[0])
    };

    let l = variable_name.len();
    ParsedEnumField::new(x.ident.to_string(), variable_name, l)
}

fn enum_data_to_tokens(data: DataEnum, main_attrs: Vec<Attribute>) -> Vec<ParsedEnumField> {
    let DataEnum { variants, .. } = data;

    let prefix_and_sufix = extract_binary_values(
        parse_hack_attrs(&main_attrs),
        &mut [("prefix", String::new()), ("suffix", String::new())],
    );

    let variants_iter = variants.iter().map(|x| {
        let variable_names = extract_binary_values(
            parse_hack_attrs(&x.attrs),
            &mut [(
                "symbol",
                x.ident
                    .span()
                    .source_text()
                    .expect("Error parsing a variant's span"),
            )],
        );

        let variable_name = if variable_names[0].len() == 0 {
            "".to_owned()
        } else {
            format!(
                "{}{}{}",
                prefix_and_sufix[0], variable_names[0], prefix_and_sufix[1]
            )
        };

        let l = variable_name.len();
        ParsedEnumField::new(x.ident.to_string(), variable_name, l)
    });
    let mut data: Vec<ParsedEnumField> = variants_iter.collect();
    data.sort_by(|a, b| b.lit_len.cmp(&a.lit_len));
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_c_instruction_test() {
        assert_eq!(
            normolize_string(parse_c_instraction("M=M+1")),
            normolize_string(
                "Instruction::C(
                    CInstruction{
                        dest: CInstructionDest::__from_static_symbols(b\"M=\"),
                        expression: CInstructionExpression::__from_static_symbols(b\"M+1\"),
                        jump: CInstructionJump::__from_static_symbols(b\"\"),
                    }
                )"
                .to_owned()
            )
        );
    }

    #[test]
    fn parse_label_instruction_test() {
        assert_eq!(
            normolize_string(parse_helper_instraction("(MYFILE_TRUE_3)")),
            normolize_string(
                "Instruction::Helper(
                    HelperInstruction::Label(
                        LabelInstruction{
                            prefix: b\"MYFILE\",
                            name: b\"TRUE\",
                            prefix_len: 6,
                            name_len: 4,
                            idx: 3,
                        }
                    )
                )"
                .to_owned()
            )
        );
    }

    fn normolize_string(s: String) -> String {
        s.replace(" ", "")
            .replace("\n", "")
            .replace(":", ": ")
            .replace(": : ", "::")
    }
}
