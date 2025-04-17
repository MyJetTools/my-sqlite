pub fn is_type_transformed(tp: &types_reader::PropertyType) -> bool {
    match tp {
        types_reader::PropertyType::DateTime => true,
        types_reader::PropertyType::VecOf(_) => true,
        types_reader::PropertyType::HashMap(_, _) => true,
        types_reader::PropertyType::Struct(_, _) => true,
        _ => false,
    }
}
