use crate::{struct_ext::StructPropertyExt, struct_schema::StructSchema};
use quote::quote;
use types_reader::PropertyType;

pub fn fn_from<'s>(
    fields: &'s impl StructSchema<'s>,
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let fields = fields.get_fields();
    let mut result = Vec::with_capacity(fields.len());

    for field in fields {
        let name_ident = field.get_field_name_ident();

        let db_column_name = field.get_db_column_name()?.to_column_name_token();

        let metadata = field.get_field_metadata()?;

        let reading = if let PropertyType::OptionOf(sub_prop) = &field.ty {
            let type_ident = sub_prop.get_token_stream_with_generics();
            quote!(#type_ident::from_db_row_opt(db_row, #db_column_name, &#metadata))
        } else {
            let type_ident = field.ty.get_token_stream_with_generics();
            quote!(#type_ident::from_db_row(db_row, #db_column_name, &#metadata))
        };

        result.push(quote! {
            #name_ident: #reading,
        });
    }

    Ok(result)
}
