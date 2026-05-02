use specta::{
    Type, Types,
    datatype::{
        DataType, Fields, NamedFields, Primitive as PrimitiveType, Reference, Struct, Tuple,
        UnnamedFields,
    },
};
use std::fmt::Write;

pub trait LuauTypeExt {
    fn luau_name() -> String;
    fn luau_definition() -> String;
}

impl<T: Type> LuauTypeExt for T {
    fn luau_name() -> String {
        let mut types = Types::default();
        let dt = T::definition(&mut types);

        match &dt {
            DataType::Reference(Reference::Named(named_ref)) => {
                named_ref_name(named_ref, &types).unwrap_or_else(|| rust_type_name::<T>())
            }
            _ => rust_type_name::<T>(),
        }
    }

    fn luau_definition() -> String {
        let mut types = Types::default();
        let dt = T::definition(&mut types);

        map_specta_to_luau(&dt, &types)
    }
}

fn rust_type_name<T: ?Sized>() -> String {
    std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .unwrap_or("UnknownType")
        .to_string()
}

fn named_ref_name(named_ref: &specta::datatype::NamedReference, types: &Types) -> Option<String> {
    named_ref.get(types).map(|named| named.name().to_string())
}

fn map_specta_to_luau(dt: &DataType, types: &Types) -> String {
    #[allow(clippy::match_wildcard_for_single_variants)]
    match dt {
        DataType::Primitive(p) => match p {
            PrimitiveType::bool => "boolean".into(),
            PrimitiveType::char | PrimitiveType::str => "string".into(),

            PrimitiveType::i8
            | PrimitiveType::i16
            | PrimitiveType::i32
            | PrimitiveType::i64
            | PrimitiveType::i128
            | PrimitiveType::isize
            | PrimitiveType::u8
            | PrimitiveType::u16
            | PrimitiveType::u32
            | PrimitiveType::u64
            | PrimitiveType::u128
            | PrimitiveType::usize
            | PrimitiveType::f16
            | PrimitiveType::f32
            | PrimitiveType::f64
            | PrimitiveType::f128 => "number".into(),
        },

        DataType::List(list) => {
            format!("{{ {} }}", map_specta_to_luau(list.ty(), types))
        }

        DataType::Map(map) => {
            format!(
                "{{ [{}]: {} }}",
                map_specta_to_luau(map.key_ty(), types),
                map_specta_to_luau(map.value_ty(), types)
            )
        }

        DataType::Struct(s) => struct_to_luau(s, types),

        DataType::Tuple(t) => tuple_to_luau(t, types),

        DataType::Nullable(inner) => {
            format!("{}?", map_specta_to_luau(inner, types))
        }

        DataType::Reference(r) => reference_to_luau(r, types),

        _ => "any".into(),
    }
}

fn reference_to_luau(reference: &Reference, types: &Types) -> String {
    match reference {
        Reference::Named(named_ref) => named_ref.get(types).map_or_else(
            || "any".into(),
            |named| map_specta_to_luau(named.ty(), types),
        ),
        Reference::Generic(_) | Reference::Opaque(_) => "any".into(),
    }
}

fn struct_to_luau(s: &Struct, types: &Types) -> String {
    match s.fields() {
        Fields::Unit => "{}".to_string(),
        Fields::Named(named) => named_fields_to_luau(named, types),
        Fields::Unnamed(unnamed) => unnamed_fields_to_luau(unnamed, types),
    }
}

fn named_fields_to_luau(fields: &NamedFields, types: &Types) -> String {
    let mut out = String::from("{ ");

    for (name, field) in fields.fields() {
        let ty = field
            .ty()
            .map_or_else(|| "any".to_string(), |ty| map_specta_to_luau(ty, types));

        if field.optional() {
            let _ = write!(out, "{name}?: {ty}, ");
        } else {
            let _ = write!(out, "{name}: {ty}, ");
        }
    }

    out.push('}');
    out
}

fn unnamed_fields_to_luau(fields: &UnnamedFields, types: &Types) -> String {
    let mut out = String::from("{ ");

    for field in fields.fields() {
        let ty = field
            .ty()
            .map_or_else(|| "any".to_string(), |ty| map_specta_to_luau(ty, types));

        let _ = write!(out, "{ty},");
    }

    out.push('}');
    out
}

fn tuple_to_luau(tuple: &Tuple, types: &Types) -> String {
    let elems = tuple.elements();

    if elems.is_empty() {
        return "nil".to_string();
    }

    let inner = elems
        .iter()
        .map(|ty| map_specta_to_luau(ty, types))
        .collect::<Vec<_>>()
        .join(", ");

    format!("{{ {inner} }}")
}
