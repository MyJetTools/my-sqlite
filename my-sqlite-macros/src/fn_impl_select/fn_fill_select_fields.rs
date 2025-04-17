use types_reader::PropertyType;

use crate::{struct_ext::StructPropertyExt, struct_schema::StructSchema};
use quote::quote;

pub fn fn_fill_select_fields<'s>(
    fields: &'s impl StructSchema<'s>,
) -> Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>), syn::Error> {
    let fields = fields.get_fields();
    let mut result = Vec::with_capacity(fields.len() * 2);

    let mut result_2 = Vec::with_capacity(fields.len() * 2);

    for prop in fields {
        if prop.is_line_no() {
            continue;
        }

        if let Ok(sql) = prop.attrs.get_single_or_named_param("sql", "sql") {
            let attr_value = sql.as_string()?.as_str();
            result.push(quote! {
                sql.push_str(#attr_value);
            });
        } else {
            let db_column_name = prop.get_db_column_name()?;
            let db_row_column_name = db_column_name.get_db_row_column_name();
            let db_row_column_name = db_row_column_name.as_str();

            let db_column_name = db_column_name.to_column_name_token();

            let metadata = prop.get_field_metadata()?;

            if let PropertyType::OptionOf(sub_type) = &prop.ty {
                let type_ident = sub_type.get_token_stream_with_generics();

                result.push(
                    quote! {
                        #type_ident::fill_select_part(sql, #db_column_name, &#metadata);
                    }
                    .into(),
                );
            } else {
                let type_ident = prop.ty.get_token_stream_with_generics();
                result.push(
                    quote! {
                        #type_ident::fill_select_part(sql, #db_column_name, &#metadata);
                    }
                    .into(),
                );
            }

            result_2.push(quote!(#db_row_column_name,));
        }
    }

    Ok((result, result_2))
}
