use proc_macro::TokenStream;
use quote::quote;
use types_reader::{StructProperty, StructureSchema};

use crate::struct_ext::StructPropertyExt;

use super::insert_fields::InsertFields;

pub fn generate(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let structure_schema = StructureSchema::new(ast)?;

    let fields = InsertFields::new(&structure_schema);

    let fields_amount = fields.get_fields_amount();

    let fn_get_column_name = fn_get_column_name(fields.as_slice())?;

    let get_field_value = fn_get_field_value(fields.as_slice())?;

    let name = structure_schema.name.get_name_ident();

    let result = quote! {
        impl my_sqlite::sql_insert::SqlInsertModel for #name{

            fn get_fields_amount()->usize{
                #fields_amount
            }

            fn get_column_name(no: usize) -> my_sqlite::ColumnName{
                match no{
                    #(#fn_get_column_name)*
                    _=>panic!("no such field with number {}", no)
                }
            }

            fn get_field_value(&self, no: usize) -> my_sqlite::sql_update::SqlUpdateModelValue{
                match no{
                    #(#get_field_value)*
                    _=>panic!("no such field with number {}", no)
                }
            }


        }

    }
    .into();

    Ok(result)
}

pub fn fn_get_column_name(
    fields: &[&StructProperty],
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut result = Vec::new();
    for (i, prop) in fields.iter().enumerate() {
        let field_name = prop.get_db_column_name()?;
        let field_name = field_name.as_str();

        result.push(quote! (#i=>#field_name.into(),).into());
    }
    Ok(result)
}

pub fn fn_get_field_value(
    fields: &[&StructProperty],
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut result = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let value = field.render_field_value(true)?;

        result.push(quote! (#i => #value,));
    }
    Ok(result)
}
