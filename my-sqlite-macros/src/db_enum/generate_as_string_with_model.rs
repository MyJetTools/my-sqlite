use quote::quote;
use types_reader::EnumCase;

use super::enum_case_ext::EnumCaseExt;

pub fn generate_as_string_with_model(ast: &syn::DeriveInput) -> Result<proc_macro::TokenStream, syn::Error> {
    let enum_name = &ast.ident;

    let enum_cases =  EnumCase::read(ast)?;

    let fn_to_str =  generate_fn_to_str(&enum_cases)?;

    let fn_from_str =  generate_fn_from_str(&enum_cases)?;


    let update_value_provider_fn_body = super::utils::render_update_value_provider_fn_body_as_json();

    let select_part = super::utils::render_select_part_as_json();


    let result = quote! {

        impl #enum_name{

     
            pub fn to_str(&self)->String {
                match self{
                    #fn_to_str
                }
            }


            pub fn from_str(src: &str)->Self{
                let (name, model) = my_sqlite::utils::get_case_and_model(src);
                match name {
                    #fn_from_str
                  _ => panic!("Invalid value {}", name)
                }
            }

            pub fn fill_select_part(sql: &mut my_sqlite::sql::SelectBuilder, field_name: my_sqlite::sql_select::DbColumnName, metadata: &Option<my_sqlite::SqlValueMetadata>) {
                #select_part
            }
        }

            impl<'s> my_sqlite::sql_select::FromDbRow<'s, #enum_name> for #enum_name{
                fn from_db_row(row: &'s my_sqlite::DbRow, name: &str, metadata: &Option<my_sqlite::SqlValueMetadata>) -> Self{
                    let value: String = row.get(name);
                    Self::from_str(value.as_str())
                }

                fn from_db_row_opt(row: &'s my_sqlite::DbRow, name: &str, metadata: &Option<my_sqlite::SqlValueMetadata>) -> Option<Self>{
                    let value: Option<String> = row.get(name);
                    let value = value?;

                    Some(Self::from_str(value.as_str()))
                }
            }

            impl my_sqlite::sql_update::SqlUpdateValueProvider for #enum_name{
                fn get_update_value(
                    &self,
                    params: &mut my_sqlite::sql::SqlValues,
                    metadata: &Option<my_sqlite::SqlValueMetadata>,
                )->my_sqlite::sql::SqlUpdateValue {
                    #update_value_provider_fn_body
                }

            }

            impl my_sqlite::table_schema::SqlTypeProvider for #enum_name {
                fn get_sql_type(
                    _metadata: Option<my_sqlite::SqlValueMetadata>,
                ) -> my_sqlite::table_schema::TableColumnType {
                    my_sqlite::table_schema::TableColumnType::Jsonb
                }
            }

    
    }
    .into();

    Ok(result)
}

fn generate_fn_from_str(enum_cases: &[EnumCase]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut result = proc_macro2::TokenStream::new();
    for case in enum_cases {
        let case_ident = &case.get_name_ident();

        let case_value = case.get_value()?.get_value_as_str();
        let case_value = case_value.as_str();

        if case.model.is_none() {
            return Err(syn::Error::new_spanned(
                case_value,
                "Model is not defined for this enum case",
            ));
        }

        let model = case.model.as_ref().unwrap().get_name_ident();

        result.extend(quote! {
            #case_value => Self::#case_ident(#model::from_str(model)),
        });
    }
    Ok(result)
}

fn generate_fn_to_str(enum_cases: &[EnumCase]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut result = proc_macro2::TokenStream::new();
    for case in enum_cases {
        let case_ident = &case.get_name_ident();

        let case_value = case.get_value()?.get_value_as_str();
        let case_value = case_value.as_str();

        result.extend(quote! {
            Self::#case_ident(model) => my_sqlite::utils::compile_enum_with_model(#case_value, model.to_string().as_str()),
        });
    }
    Ok(result)
}
