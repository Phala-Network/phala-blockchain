use either::Either;
use heck::{CamelCase, SnakeCase};
use itertools::Itertools;
use log::debug;
use multimap::MultiMap;
use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::source_code_info::Location;
use prost_types::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet, SourceCodeInfo,
};
use std::collections::HashMap;

#[derive(PartialEq)]
enum Syntax {
    Proto2,
    Proto3,
}

pub struct CodeGenerator<'a> {
    package: String,
    source_info: SourceCodeInfo,
    syntax: Syntax,
    depth: u8,
    path: Vec<i32>,
    buf: &'a mut String,
    mod_path: Vec<String>,
    mod_prefix: String,
    type_prefix: String,
}

impl<'a> CodeGenerator<'a> {
    pub fn generate(
        file: FileDescriptorProto,
        buf: &mut String,
        mod_prefix: impl Into<String>,
        type_prefix: impl Into<String>,
    ) {
        let mut source_info = file
            .source_code_info
            .expect("no source code info in request");
        source_info.location.retain(|location| {
            let len = location.path.len();
            len > 0 && len % 2 == 0
        });
        source_info
            .location
            .sort_by_key(|location| location.path.clone());

        let syntax = match file.syntax.as_ref().map(String::as_str) {
            None | Some("proto2") => Syntax::Proto2,
            Some("proto3") => Syntax::Proto3,
            Some(s) => panic!("unknown syntax: {}", s),
        };

        let mut code_gen = CodeGenerator {
            package: file.package.unwrap(),
            source_info,
            syntax,
            depth: 0,
            path: Vec::new(),
            buf,
            mod_path: vec![],
            mod_prefix: mod_prefix.into(),
            type_prefix: type_prefix.into(),
        };

        debug!(
            "file: {:?}, package: {:?}",
            file.name.as_ref().unwrap(),
            code_gen.package
        );

        code_gen.path.push(4);
        for (idx, message) in file.message_type.into_iter().enumerate() {
            code_gen.path.push(idx as i32);
            code_gen.append_message(message);
            code_gen.path.pop();
        }
        code_gen.path.pop();
    }

    fn append_message(&mut self, message: DescriptorProto) {
        debug!("  message: {:?}", message.name());

        let mut impl_buf = String::new();
        if self.append_message_impl(message.clone(), &mut impl_buf) {
            self.buf.push_str(&impl_buf);
        }

        let message_name = message.name().to_string();
        let fq_message_name = format!(".{}.{}", self.package, message.name());

        // Split the nested message types into a vector of normal nested message types, and a map
        // of the map field entry types. The path index of the nested message types is preserved so
        // that comments can be retrieved.
        type NestedTypes = Vec<(DescriptorProto, usize)>;
        type MapTypes = HashMap<String, (FieldDescriptorProto, FieldDescriptorProto)>;
        let (nested_types, _): (NestedTypes, MapTypes) = message
            .nested_type
            .into_iter()
            .enumerate()
            .partition_map(|(idx, nested_type)| {
                if nested_type
                    .options
                    .as_ref()
                    .and_then(|options| options.map_entry)
                    .unwrap_or(false)
                {
                    let key = nested_type.field[0].clone();
                    let value = nested_type.field[1].clone();
                    assert_eq!("key", key.name());
                    assert_eq!("value", value.name());

                    let name = format!("{}.{}", &fq_message_name, nested_type.name());
                    Either::Right((name, (key, value)))
                } else {
                    Either::Left((nested_type, idx))
                }
            });
        if !nested_types.is_empty() {
            self.push_mod(&message_name);
            self.path.push(3);
            for (nested_type, idx) in nested_types {
                self.path.push(idx as i32);
                self.append_message(nested_type);
                self.path.pop();
            }
            self.path.pop();
            self.pop_mod();
        }
    }

    fn append_field(&mut self, field: FieldDescriptorProto, buf: &mut String) -> bool {
        let optional = self.optional(&field);
        let ty = self.resolve_decoded_type(&field);

        debug!("    field: {:?}, type: {:?}", field.name(), ty,);

        if let Some((name, _type_path)) = self.codec_decoration() {
            match name.as_str() {
                "scale" => {
                    buf.push_str(&format!(
                        "pub fn {}_decoded(&self) -> Result<{}, ScaleDecodeError> {{\n",
                        field.name(),
                        ty
                    ));
                    if optional {
                        buf.push_str(&format!(
                            "self.{}.as_ref().map(|v| Decode::decode(&mut &v[..])).transpose()",
                            field.name()
                        ));
                    } else {
                        buf.push_str(&format!(
                            "::prpc::codec::scale::Decode::decode(&mut &self.{}[..])",
                            field.name()
                        ));
                    }
                    buf.push_str("\n}\n");
                    return true;
                }
                _ => {
                    panic!("Unknown codec {}", name);
                }
            }
        }
        false
    }

    fn append_message_impl(&mut self, message: DescriptorProto, buf: &mut String) -> bool {
        let message_name = message.name().to_string();
        // Split the fields into a vector of the normal fields, and oneof fields.
        // Path indexes are preserved so that comments can be retrieved.
        type Fields = Vec<(FieldDescriptorProto, usize)>;
        type OneofFields = MultiMap<i32, (FieldDescriptorProto, usize)>;
        let (fields, _): (Fields, OneofFields) = message
            .field
            .into_iter()
            .enumerate()
            .partition_map(|(idx, field)| {
                if field.proto3_optional.unwrap_or(false) {
                    Either::Left((field, idx))
                } else if let Some(oneof_index) = field.oneof_index {
                    Either::Right((oneof_index, (field, idx)))
                } else {
                    Either::Left((field, idx))
                }
            });

        buf.push_str("impl ");
        let mut msg_path = self.mod_path.clone();
        msg_path.push(to_upper_camel(&message_name));
        let msg_path = self.mod_prefix.clone() + &msg_path.join("::");
        buf.push_str(&msg_path);
        buf.push_str(" {\n");

        self.path.push(2);
        let mut n_fields = 0;
        for (field, idx) in fields.clone() {
            self.path.push(idx as i32);
            if self.append_field(field, buf) {
                n_fields += 1;
            }
            self.path.pop();
        }
        self.path.pop();

        if n_fields > 0 {
            buf.push_str("pub fn new(\n");
            self.path.push(2);
            for (field, idx) in fields.clone() {
                self.path.push(idx as i32);
                buf.push_str(&format!(
                    "{}: {},\n",
                    field.name(),
                    self.resolve_decoded_type(&field)
                ));
                self.path.pop();
            }
            self.path.pop();
            buf.push_str(") -> Self {\n");
            buf.push_str("  Self{\n");
            self.path.push(2);
            for (field, idx) in fields.clone() {
                self.path.push(idx as i32);
                if let Some(_) = self.codec_decoration() {
                    if self.optional(&field) {
                        buf.push_str(&format!(
                            "{}: {}.map(|x| x.encode()),\n",
                            field.name(),
                            field.name()
                        ));
                    } else {
                        buf.push_str(&format!("{}: {}.encode(),\n", field.name(), field.name()));
                    }
                } else {
                    buf.push_str(&format!("{},\n", field.name()));
                }
                self.path.pop();
            }
            self.path.pop();
            buf.push_str("  }\n");
            buf.push_str("}\n");
        }

        buf.push_str("\n}\n");
        n_fields > 0
    }

    fn location(&self) -> &Location {
        let idx = self
            .source_info
            .location
            .binary_search_by_key(&&self.path[..], |location| &location.path[..])
            .unwrap();

        &self.source_info.location[idx]
    }

    fn codec_decoration(&self) -> Option<(String, String)> {
        let comments = self.location().leading_comments();
        comments.split("\n").find_map(|line| {
            let line = line.trim_start();
            let parts: Vec<_> = line.split_whitespace().collect();
            match parts[..] {
                ["@codec", name, type_path] => Some((name.to_owned(), type_path.to_owned())),
                _ => None,
            }
        })
    }

    fn push_mod(&mut self, module: &str) {
        self.mod_path.push(to_snake(module));
        self.depth += 1;
    }

    fn pop_mod(&mut self) {
        self.depth -= 1;
        self.mod_path.pop();
    }

    fn optional(&self, field: &FieldDescriptorProto) -> bool {
        if field.proto3_optional.unwrap_or(false) {
            return true;
        }

        if field.label() != Label::Optional {
            return false;
        }

        match field.r#type() {
            Type::Message => true,
            _ => self.syntax == Syntax::Proto2,
        }
    }

    fn resolve_decoded_type(&self, field: &FieldDescriptorProto) -> String {
        if let Some((_name, type_path)) = self.codec_decoration() {
            let type_path = self.type_prefix.clone() + type_path.as_str();
            if self.optional(field) {
                return format!("Option<{}>", type_path);
            } else {
                return type_path;
            }
        }
        self.resolve_type(field)
    }

    fn resolve_type(&self, field: &FieldDescriptorProto) -> String {
        let ty = match field.r#type() {
            Type::Float => String::from("f32"),
            Type::Double => String::from("f64"),
            Type::Uint32 | Type::Fixed32 => String::from("u32"),
            Type::Uint64 | Type::Fixed64 => String::from("u64"),
            Type::Int32 | Type::Sfixed32 | Type::Sint32 | Type::Enum => String::from("i32"),
            Type::Int64 | Type::Sfixed64 | Type::Sint64 => String::from("i64"),
            Type::Bool => String::from("bool"),
            Type::String => String::from("::prost::alloc::string::String"),
            Type::Bytes => String::from("::prost::alloc::vec::Vec<u8>"),
            Type::Group | Type::Message => self.resolve_ident(field.type_name()),
        };
        if self.optional(field) {
            format!("Option<{}>", ty)
        } else {
            ty
        }
    }

    fn resolve_ident(&self, pb_ident: &str) -> String {
        // protoc should always give fully qualified identifiers.
        assert_eq!(".", &pb_ident[..1]);

        let mut local_path = self.package.split('.').peekable();

        let mut ident_path = pb_ident[1..].split('.');
        let ident_type = ident_path.next_back().unwrap();
        let mut ident_path = ident_path.peekable();

        // Skip path elements in common.
        while local_path.peek().is_some() && local_path.peek() == ident_path.peek() {
            local_path.next();
            ident_path.next();
        }

        local_path
            .map(|_| "super".to_string())
            .chain(ident_path.map(to_snake))
            .chain(std::iter::once(to_upper_camel(ident_type)))
            .join("::")
    }
}

/// Converts a `camelCase` or `SCREAMING_SNAKE_CASE` identifier to a `lower_snake` case Rust field
/// identifier.
pub fn to_snake(s: &str) -> String {
    let mut ident = s.to_snake_case();

    // Use a raw identifier if the identifier matches a Rust keyword:
    // https://doc.rust-lang.org/reference/keywords.html.
    match ident.as_str() {
        // 2015 strict keywords.
        | "as" | "break" | "const" | "continue" | "else" | "enum" | "false"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
        | "pub" | "ref" | "return" | "static" | "struct" | "trait" | "true"
        | "type" | "unsafe" | "use" | "where" | "while"
        // 2018 strict keywords.
        | "dyn"
        // 2015 reserved keywords.
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" | "priv" | "typeof"
        | "unsized" | "virtual" | "yield"
        // 2018 reserved keywords.
        | "async" | "await" | "try" => ident.insert_str(0, "r#"),
        // the following keywords are not supported as raw identifiers and are therefore suffixed with an underscore.
        "self" | "super" | "extern" | "crate" => ident += "_",
        _ => (),
    }
    ident
}

/// Converts a `snake_case` identifier to an `UpperCamel` case Rust type identifier.
pub fn to_upper_camel(s: &str) -> String {
    let mut ident = s.to_camel_case();

    // Suffix an underscore for the `Self` Rust keyword as it is not allowed as raw identifier.
    if ident == "Self" {
        ident += "_";
    }
    ident
}

pub fn patch_all_types(
    file_descriptor_set_path: impl AsRef<std::path::Path>,
    out_file: impl AsRef<std::path::Path>,
    mod_prefix: &str,
    type_prefix: &str,
) {
    use prost::Message;
    let buf = std::fs::read(file_descriptor_set_path).unwrap();
    let file_descriptor_set = FileDescriptorSet::decode(&*buf).unwrap();
    let mut buf = String::new();
    buf.push_str(
        r#"
    use ::prpc::codec::scale::{Encode, Decode, Error as ScaleDecodeError};
    "#,
    );
    for file in file_descriptor_set.file {
        CodeGenerator::generate(file, &mut buf, mod_prefix, type_prefix);
    }
    std::fs::write(out_file, buf).unwrap();
}
