pub fn render_where_value_provider(
    struct_name: &syn::Ident,
    render_body: impl Fn() -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let body = render_body();
    quote::quote! {
        impl my_sqlite::SqlWhereValueProvider for #struct_name{
            fn fill_where_value(
                &self,
                full_where_condition: Option<my_sqlite::RenderFullWhereCondition>,
                sql: &mut String,
                params: &mut my_sqlite::sql::SqlValues,
                metadata: &Option<my_sqlite::SqlValueMetadata>,
            )->bool{
                #body
            }

            fn render_value(&self) -> bool {
             true
            }

        }

    }
}

pub fn render_standard_operator_check(default_op: &str) -> proc_macro2::TokenStream {
    quote::quote! {
        if let Some(full_condition) = full_where_condition {
            full_condition.render_param_name(sql, #default_op, metadata)
        }
    }
}
