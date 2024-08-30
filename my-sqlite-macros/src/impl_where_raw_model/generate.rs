use std::collections::HashMap;

use sql_core::sql_with_placeholders::*;
use types_reader::{StructureSchema, TokensObject, TypeName};

use crate::{struct_ext::StructPropertyExt, where_fields::WhereFields};

pub fn generate_where_raw_model<'s>(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let attr: proc_macro2::TokenStream = attr.into();
    let params_list = TokensObject::new(attr.into())?;

    let sql = params_list.get_value_from_single_or_named("sql")?;

    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let type_name: TypeName = (&ast).try_into()?;

    let src_fields = StructureSchema::new(&ast)?;

    let where_fields = WhereFields::new(&src_fields);

    let generate_limit_fn = where_fields.generate_limit_fn();
    let generate_offset_fn = where_fields.generate_offset_fn();

    let mut src_as_hashmap = HashMap::new();

    for field in where_fields.where_fields {
        src_as_hashmap.insert(field.name.to_string(), field);
    }

    let tokens = scan_sql_for_placeholders(sql.try_into()?);

    let mut content_to_render = Vec::new();

    let mut prev_raw_content = None;

    let mut has_conditions = Vec::new();

    for token in tokens {
        match token {
            SqlTransformToken::PlaceHolder(property_name) => {
                let property = src_as_hashmap.get(property_name);

                if property.is_none() {
                    panic!("Property not found: {}", property_name)
                }

                let property = property.unwrap();

                let name = property.get_field_name_ident();
                let meta_data = property.get_field_metadata()?;

                content_to_render.push(quote::quote!(

                    if self.#name.render_value(){
                        #prev_raw_content
                        self.#name.fill_where_value(None, sql, params, &#meta_data);
                    }

                ));

                if has_conditions.len() > 0 {
                    has_conditions.push(quote::quote! { && });
                } else {
                    has_conditions.push(quote::quote! { use my_sqlite::SqlWhereValueProvider; });
                }
                has_conditions.push(quote::quote! {
                    self.#name.render_value()
                });

                prev_raw_content = None;
            }
            SqlTransformToken::RawContent(content) => {
                prev_raw_content = Some(quote::quote!(
                    sql.push_str(#content);
                ));
            }
        }
    }

    if let Some(prev_raw_content) = prev_raw_content {
        content_to_render.push(prev_raw_content);
    }

    let impl_where_model = crate::render_impl::impl_sql_where_model(
        &type_name,
        {
            quote::quote! {
                use my_sqlite::SqlWhereValueProvider;
                #(#content_to_render)*
            }
        },
        quote::quote!(#(#has_conditions)*),
        generate_limit_fn,
        generate_offset_fn,
    );

    Ok(quote::quote! {
        #ast
        #impl_where_model
    })
}
