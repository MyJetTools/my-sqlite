use types_reader::StructProperty;

use crate::struct_schema::StructSchema;

pub struct InsertFields<'s> {
    items: Vec<&'s StructProperty<'s>>,
}

impl<'s> InsertFields<'s> {
    pub fn new(src: &'s impl StructSchema<'s>) -> Self {
        Self {
            items: src.get_fields(),
        }
    }

    pub fn get_fields_amount(&self) -> usize {
        self.items.len()
    }

    pub fn as_slice(&'s self) -> &'s [&'s StructProperty<'s>] {
        self.items.as_slice()
    }
}
