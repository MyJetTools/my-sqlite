use proc_macro2::TokenStream;
use quote::quote;
use types_reader::EnumCase;

use super::enum_case_ext::EnumCaseExt;


pub enum EnumType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
}

impl EnumType {
    pub fn get_func_name(&self) -> proc_macro2::TokenStream {
        match self {
            EnumType::U8 => quote!(to_u8).into(),
            EnumType::I8 => quote!(to_i8).into(),
            EnumType::U16 => quote!(to_u16).into(),
            EnumType::I16 => quote!(to_i16).into(),
            EnumType::U32 => quote!(to_u32).into(),
            EnumType::I32 => quote!(to_i32).into(),
            EnumType::U64 => quote!(to_u64).into(),
            EnumType::I64 => quote!(to_i64).into(),
        }
    }

    pub fn get_return_type_name(&self) -> proc_macro2::TokenStream {
        match self {
            EnumType::U8 => quote!(u8).into(),
            EnumType::I8 => quote!(i8).into(),
            EnumType::U16 => quote!(u16).into(),
            EnumType::I16 => quote!(i16).into(),
            EnumType::U32 => quote!(u32).into(),
            EnumType::I32 => quote!(i32).into(),
            EnumType::U64 => quote!(u64).into(),
            EnumType::I64 => quote!(i64).into(),
        }
    }

    pub fn get_compliant_with_db_type(&self) -> proc_macro2::TokenStream {
        match self {
            EnumType::U8 => quote!(i32).into(),
            EnumType::I8 => quote!(i32).into(),
            EnumType::U16 => quote!(i32).into(),
            EnumType::I16 => quote!(i32).into(),
            EnumType::U32 => quote!(i32).into(),
            EnumType::I32 => quote!(i32).into(),
            EnumType::U64 => quote!(i64).into(),
            EnumType::I64 => quote!(i32).into(),
        }
    }
}

pub fn generate(
    ast: &syn::DeriveInput,
    enum_type: EnumType,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let enum_name = &ast.ident;
    let enum_cases = EnumCase::read(ast)?;

    for enum_case in &enum_cases {
        enum_case
            .attrs
            .check_for_unknown_params(|attr_name, params| match attr_name {
                "enum_case" => params.check_for_unknown_params(&["id", "value"]),
                _ => Ok(()),
            })?;
    }

    let to_func_name = enum_type.get_func_name();

    let type_name = enum_type.get_return_type_name();

    let as_numbered = fn_as_numbered_str(enum_cases.as_slice())?;

    let from_db_value = fn_from_db_value(enum_cases.as_slice())?;

    let to_typed_number = fn_to_typed_number(enum_cases.as_slice())?;

    let sql_db_type = enum_type.get_compliant_with_db_type();

    let from_db_result = if type_name.to_string() == sql_db_type.to_string() {
        quote! {
            Self::from_db_value(result)
        }
    } else {
        quote! {
            Self::from_db_value(result as #type_name)
        }
    };

    let default_value_reading = super::utils::get_default_value( enum_cases.as_slice())?;

    let impl_where_value_provider = crate::where_value_provider::render_where_value_provider(enum_name, ||{
        let operator_check = crate::where_value_provider::render_standard_operator_check("=");
        quote::quote!{
            #operator_check
            sql.push_str(self.as_numbered_str());
            true
        }
    });
    

    let result = quote! {

        impl #enum_name{

            
            #default_value_reading

            pub fn #to_func_name(&self)->#type_name{
                match self {
                    #(#to_typed_number),*
                }
            }

            pub fn as_numbered_str(&self)->&'static str {
                match self{
                #(#as_numbered),*
                }
            }

            pub fn from_db_value(src: #type_name)->Self{
                match src{
                  #(#from_db_value)*
                  _ => panic!("Invalid value {}", src)
                }
            }

            pub fn fill_select_part(sql: &mut  my_sqlite::sql::SelectBuilder, field_name: my_sqlite::sql_select::DbColumnName, metadata: &Option<my_sqlite::SqlValueMetadata>) {
                sql.push(my_sqlite::sql::SelectFieldValue::Field(field_name));
            }

        }

        impl my_sqlite::sql_update::SqlUpdateValueProvider for #enum_name{
            fn get_update_value(
                &self,
                params: &mut my_sqlite::sql::SqlValues,
                metadata: &Option<my_sqlite::SqlValueMetadata>,
            )->my_sqlite::sql::SqlUpdateValue {
                my_sqlite::sql::SqlUpdateValue::NonStringValue(self.as_numbered_str().into())
            }
        }

        #impl_where_value_provider

        impl<'s> my_sqlite::sql_select::FromDbRow<'s, #enum_name> for #enum_name{
            fn from_db_row(row: &'s my_sqlite::DbRow, name: &str, metadata: &Option<my_sqlite::SqlValueMetadata>) -> Self{
                let result: #sql_db_type = row.get(name);
                #from_db_result
            }

            fn from_db_row_opt(row: &'s my_sqlite::DbRow, name: &str, metadata: &Option<my_sqlite::SqlValueMetadata>) -> Option<Self>{
                let result: Option<#sql_db_type> = row.get(name);
                let result = result?;
                Some(#from_db_result)
            }
        }

        impl my_sqlite::table_schema::SqlTypeProvider for #enum_name {
            fn get_sql_type(
                _metadata: Option<my_sqlite::SqlValueMetadata>,
            ) -> my_sqlite::table_schema::TableColumnType {
                use my_sqlite::table_schema::*;
                #type_name::get_sql_type(None)
            }
        }




    }
    .into();

    Ok(result)
}

fn fn_to_typed_number(enum_cases: &[EnumCase]) -> Result<Vec<TokenStream>, syn::Error> {
    let mut result = Vec::with_capacity(enum_cases.len());
    let mut no = 0;
    for enum_case in enum_cases {
        let enum_case_name = enum_case.get_name_ident();

        no = match enum_case.get_value()?.as_number()? {
            Some(value) => value,
            None => no + 1,
        };

        let no = proc_macro2::Literal::i64_unsuffixed(no);

        result.push(quote!(Self::#enum_case_name => #no));
    }

    Ok(result)
}

pub fn fn_as_numbered_str(enum_cases: &[EnumCase]) -> Result<Vec<TokenStream>, syn::Error> {
    let mut result = Vec::with_capacity(enum_cases.len());
    let mut no = 0;
    for enum_case in enum_cases {
        let enum_case_name = enum_case.get_name_ident();

        no = match enum_case.get_value()?.as_number()? {
            Some(value) => value,
            None => no + 1,
        };

        let no = no.to_string();

        result.push(quote!(Self::#enum_case_name => #no).into());
    }

    Ok(result)
}

fn fn_from_db_value(enum_cases: &[EnumCase]) -> Result<Vec<TokenStream>, syn::Error> {
    let mut result = Vec::with_capacity(enum_cases.len());

    let mut no = 0;

    for enum_case in enum_cases {
        no = match enum_case.get_value()?.as_number()? {
            Some(value) => value,
            None => no + 1,
        };

        let no = proc_macro2::Literal::i64_unsuffixed(no);

        let name_ident = enum_case.get_name_ident();

        result.push(quote! (#no => Self::#name_ident,));
    }

    Ok(result)
}
