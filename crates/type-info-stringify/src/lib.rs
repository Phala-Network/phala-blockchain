use scale_info::{form::PortableForm, IntoPortable, PortableRegistry, TypeDef, TypeInfo};

pub fn type_info_stringify<T: TypeInfo>() -> String {
    let mut registry = Default::default();
    let root = T::type_info().into_portable(&mut registry);
    let registry = PortableRegistry::from(registry);

    for (i, ty) in registry.types.iter().enumerate() {
        assert_eq!(i, ty.id as usize);
    }

    let mut output = String::new();
    for ty in &registry.types {
        if ty.ty.path.segments.is_empty() {
            continue;
        }
        output.push_str(&format!(
            "{} = {}\n",
            ty.ty.path,
            type_def_of(&registry, &ty.ty.type_def)
        ));
    }
    output.push_str(&format!(
        "{} = {}\n",
        root.path,
        &type_def_of(&registry, &root.type_def)
    ));
    output
}

fn opt(v: &Option<impl AsRef<str>>) -> &str {
    match v {
        None => "",
        Some(v) => v.as_ref(),
    }
}

fn resolve_type(registry: &PortableRegistry, id: u32) -> String {
    let Some(t) = registry.types.get(id as usize) else {
        return format!("<unknown type {id}>");
    };
    let mut name = if !t.ty.path.segments.is_empty() {
        t.ty.path.to_string()
    } else {
        type_def_of(registry, &t.ty.type_def)
    };
    if !t.ty.type_params.is_empty() {
        name.push('<');
        for (i, param) in t.ty.type_params.iter().enumerate() {
            if i > 0 {
                name.push_str(",");
            }
            match param.ty {
                Some(t) => name.push_str(&resolve_type(registry, t.id)),
                None => name.push_str(&param.name),
            }
        }
        name.push('>');
    }
    name
}

fn type_def_of(registry: &PortableRegistry, type_def: &TypeDef<PortableForm>) -> String {
    let type_of = |id: u32| resolve_type(registry, id);
    let mut s = String::new();
    match type_def {
        TypeDef::Composite(def) => {
            s.push_str("struct {\n");
            for field in &def.fields {
                s.push_str(&format!(
                    "    {}: {},\n",
                    opt(&field.name),
                    type_of(field.ty.id),
                ));
            }
            s.push_str("}");
        }
        TypeDef::Variant(def) => {
            s.push_str("enum {\n");
            for variant in &def.variants {
                s.push_str(format!("    [{}]{}", variant.index, variant.name).as_str());
                if variant.fields.is_empty() {
                    s.push_str(",\n");
                } else if variant.fields[0].name.is_none() {
                    s.push_str("(");
                    for (i, field) in variant.fields.iter().enumerate() {
                        if i > 0 {
                            s.push_str(", ");
                        }
                        s.push_str(&format!("{}", type_of(field.ty.id)));
                    }
                    s.push_str(")\n");
                } else {
                    s.push_str(" {\n");
                    for field in &variant.fields {
                        s.push_str(&format!(
                            "        {}: {},\n",
                            opt(&field.name),
                            type_of(field.ty.id),
                        ));
                    }
                    s.push_str("    }\n");
                }
            }
            s.push_str("}");
        }
        TypeDef::Sequence(def) => {
            s.push_str(format!("Vec<{}>", type_of(def.type_param.id)).as_str());
        }
        TypeDef::Array(def) => {
            s.push_str(&format!("[{}; {}]", type_of(def.type_param.id), def.len));
        }
        TypeDef::Tuple(def) => {
            s.push_str("(");
            for (i, field) in def.fields.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!("{}", type_of(field.id)));
            }
            s.push_str(")");
        }
        TypeDef::Primitive(def) => {
            s.push_str(&format!("{:?}", def).to_lowercase());
        }
        TypeDef::Compact(def) => {
            s.push_str(&format!("Compact<{}>", type_of(def.type_param.id)));
        }
        TypeDef::BitSequence(def) => {
            s.push_str(&format!(
                "BitsOf<{},{}>",
                type_of(def.bit_store_type.id),
                type_of(def.bit_order_type.id),
            ));
        }
    }
    s
}
