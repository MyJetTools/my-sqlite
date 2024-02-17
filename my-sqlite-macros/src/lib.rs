mod attributes;
mod db_enum;
mod fn_impl_insert;
mod fn_impl_select;
mod fn_impl_update;
mod fn_impl_where_model;
mod render_impl;
mod struct_ext;
mod struct_schema;
mod table_schema;
mod where_fields;
mod where_value_provider;

use proc_macro::TokenStream;

#[proc_macro_derive(
    TableSchema,
    attributes(
        bigint,
        sql_type,
        ignore_table_column,
        primary_key,
        db_index,
        default_if_null,
        default_value,
        wrap_column_name,
        db_column_name,
        generate_select_model,
        generate_update_model,
        generate_where_model,
    )
)]
pub fn table_schema(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let result = crate::table_schema::generate(&ast);

    match result {
        Ok(result) => {
            #[cfg(feature = "debug-table-schema")]
            println!("{}", result);
            result
        }
        Err(e) => e.into_compile_error().into(),
    }
}

#[proc_macro_derive(
    UpdateDbEntity,
    attributes(
        db_column_name,
        primary_key,
        ignore,
        sql_type,
        e_tag,
        default_if_null,
        ignore_if_none,
        wrap_column_name,
        json,
    )
)]
pub fn update_db_entity(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match crate::fn_impl_update::generate(&ast) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(
    SelectDbEntity,
    attributes(
        db_column_name,
        line_no,
        sql,
        sql_type,
        order_by,
        order_by_desc,
        group_by,
        primary_key,
        default_if_null,
        wrap_column_name,
    )
)]
pub fn select_db_entity(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match crate::fn_impl_select::generate(&ast) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(
    InsertDbEntity,
    attributes(
        db_column_name,
        ignore,
        bigint,
        json,
        sql_type,
        primary_key,
        e_tag,
        default_if_null,
        ignore_if_none,
        wrap_column_name,
    )
)]
pub fn insert_db_entity(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match crate::fn_impl_insert::generate(&ast) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(
    WhereDbModel,
    attributes(
        db_column_name,
        bigint,
        operator,
        ignore_if_none,
        ignore,
        limit,
        offset,
        sql_type,
        default_if_null,
        wrap_column_name,
        inside_json
    )
)]
pub fn where_db_model(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match crate::fn_impl_where_model::generate(&ast) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(DbEnumAsString, attributes(enum_case, default_if_null, default_value,))]
pub fn db_enum_as_string(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    match crate::db_enum::generate_as_string(&ast) {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into(),
    }
}
