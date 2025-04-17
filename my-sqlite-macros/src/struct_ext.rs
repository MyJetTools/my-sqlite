use proc_macro2::TokenStream;
use types_reader::{rust_extensions::StrOrString, MacrosAttribute, PropertyType, StructProperty};

use crate::attributes::*;

pub struct DbColumnName<'s> {
    pub attr: Option<DbColumnNameAttribute<'s>>,
    pub property_name: &'s str,
    pub force_cast_db_type: bool,
    pub ty: &'s PropertyType<'s>,
}

impl<'s> DbColumnName<'s> {
    pub fn to_column_name_token(&'s self) -> proc_macro2::TokenStream {
        let db_column_name = match self.attr.as_ref() {
            Some(attr) => attr.name,
            None => self.property_name,
        };

        let filed_name = self.property_name;

        let force_cast_db_type = self.force_cast_db_type;
        quote::quote! {
            my_sqlite::sql_select::DbColumnName{
                field_name: #filed_name,
                db_column_name: #db_column_name,
                force_cast_db_type: #force_cast_db_type
            }
        }
    }

    pub fn as_str(&self) -> &str {
        if let Some(attr) = &self.attr {
            return attr.name;
        }

        self.property_name
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn get_db_row_column_name(&'s self) -> StrOrString<'s> {
        if let Some(attr) = &self.attr {
            return attr.name.into();
        }

        if crate::utils::is_type_transformed(self.ty) {
            return format!("{}.transformed", self.property_name).into();
        }

        self.property_name.into()
    }

    pub fn get_overridden_column_name(&self) -> DbColumnNameAttribute {
        if let Some(attr) = &self.attr {
            return DbColumnNameAttribute { name: attr.name };
        }

        DbColumnNameAttribute {
            name: self.property_name,
        }
    }
}

pub enum DefaultValue {
    Inherit,
    Value(String),
}

pub trait StructPropertyExt<'s> {
    fn is_primary_key(&self) -> bool;

    fn get_db_column_name(&self) -> Result<DbColumnName, syn::Error>;

    fn has_ignore_attr(&self) -> bool;
    fn has_ignore_if_none_attr(&self) -> bool;

    fn is_line_no(&self) -> bool;

    fn get_field_metadata(&self) -> Result<proc_macro2::TokenStream, syn::Error>;

    fn has_ignore_table_column(&self) -> bool;

    fn get_ty(&self) -> &PropertyType;

    fn get_field_name_ident(&self) -> &syn::Ident;

    fn get_default_value(&self) -> Result<Option<DefaultValue>, syn::Error>;

    fn get_sql_type_as_token_stream(&self) -> Result<proc_macro2::TokenStream, syn::Error>;

    fn get_force_cast_db_type(&self) -> bool;

    fn fill_attributes(
        &self,
        fields: &mut Vec<TokenStream>,
        override_db_column_name: Option<DbColumnNameAttribute>,
    ) -> Result<(), syn::Error>;

    fn render_field_value(&self, is_update: bool) -> Result<proc_macro2::TokenStream, syn::Error> {
        match &self.get_ty() {
            types_reader::PropertyType::OptionOf(_) => return self.fill_option_of_value(is_update),
            types_reader::PropertyType::Struct(..) => return self.get_value(is_update),
            _ => return self.get_value(is_update),
        }
    }

    fn get_value(&self, is_update: bool) -> Result<proc_macro2::TokenStream, syn::Error> {
        let name = self.get_field_name_ident();

        let metadata = self.get_field_metadata()?;

        let ignore_if_none = self.has_ignore_if_none_attr();

        let result = if is_update {
            quote::quote! {
                my_sqlite::sql_update::SqlUpdateModelValue{
                    value: Some(&self.#name),
                    ignore_if_none: #ignore_if_none,
                    metadata: #metadata
                }
            }
            .into()
        } else {
            quote::quote! {
                my_sqlite::SqlWhereValueWrapper::Value {
                    value: &self.#name,
                    metadata: #metadata
                }
            }
            .into()
        };

        Ok(result)
    }

    fn fill_option_of_value(
        &self,
        is_update: bool,
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        let prop_name = self.get_field_name_ident();

        let metadata = self.get_field_metadata()?;

        let else_case: proc_macro2::TokenStream = if self.has_ignore_if_none_attr() {
            if is_update {
                quote::quote!(my_sqlite::sql_update::SqlUpdateModelValue::Ignore).into()
            } else {
                quote::quote!(my_sqlite::sql_update::SqlUpdateModelValue::Ignore).into()
            }
        } else {
            if is_update {
                quote::quote!(my_sqlite::sql_update::SqlUpdateModelValue::Null).into()
            } else {
                quote::quote!(my_sqlite::sql_update::SqlUpdateModelValue::Null).into()
            }
        };

        let result = if is_update {
            let ignore_if_none = self.has_ignore_if_none_attr();

            quote::quote! {
               if let Some(value) = &self.#prop_name{
                  my_sqlite::sql_update::SqlUpdateModelValue {value: Some(value), ignore_if_none:#ignore_if_none, metadata: #metadata}
               }else{
                my_sqlite::sql_update::SqlUpdateModelValue {value: None, ignore_if_none:#ignore_if_none, metadata: #metadata}
               }
            }
        } else {
            quote::quote! {
               if let Some(value) = &self.#prop_name{
                  my_sqlite::SqlWhereValueWrapper::Value {value, metadata: #metadata}
               }else{
                    #else_case
               }
            }
        };

        Ok(result)
    }
}

impl<'s> StructPropertyExt<'s> for StructProperty<'s> {
    fn get_force_cast_db_type(&self) -> bool {
        self.attrs.try_get_attr("force_cast_db_type").is_some()
    }

    fn get_field_name_ident(&self) -> &syn::Ident {
        self.get_field_name_ident()
    }

    fn get_ty(&self) -> &PropertyType {
        &self.ty
    }

    fn is_primary_key(&self) -> bool {
        self.attrs.has_attr(PrimaryKeyAttribute::NAME)
    }

    fn get_db_column_name(&self) -> Result<DbColumnName, syn::Error> {
        let attr: Option<DbColumnNameAttribute> = self.try_get_attribute()?;
        let force_cast_db_type = self.get_force_cast_db_type();

        let result = DbColumnName {
            attr,
            property_name: &self.name,
            force_cast_db_type,
            ty: &self.ty,
        };

        Ok(result)
    }

    fn has_ignore_attr(&self) -> bool {
        self.attrs.has_attr(IgnoreAttribute::NAME)
    }

    fn has_ignore_if_none_attr(&self) -> bool {
        self.attrs.has_attr(IgnoreIfNoneAttribute::NAME)
    }

    fn has_ignore_table_column(&self) -> bool {
        self.attrs.has_attr(IgnoreTableColumnAttribute::NAME)
    }

    fn is_line_no(&self) -> bool {
        self.attrs.has_attr(LineNoAttribute::NAME) || self.name == LineNoAttribute::NAME
    }

    fn get_default_value(&self) -> Result<Option<DefaultValue>, syn::Error> {
        let default_value_attr: Option<DefaultValueAttribute> = self.try_get_attribute()?;
        if let Some(attr) = default_value_attr {
            match attr.value {
                Some(value) => {
                    return Ok(Some(DefaultValue::Value(value.to_string())));
                }
                None => return Ok(Some(DefaultValue::Inherit)),
            }
        }

        return Ok(None);
    }

    fn get_field_metadata(&self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let sql_type: Option<SqlTypeAttribute> = self.try_get_attribute()?;
        let operator: Option<WhereOperatorAttribute> = self.try_get_attribute()?;
        if sql_type.is_none() && operator.is_none() {
            return Ok(quote::quote!(None));
        }

        let sql_type = if let Some(sql_type) = sql_type {
            let sql_type = sql_type.name.as_str();
            quote::quote!(Some(#sql_type))
        } else {
            quote::quote!(None)
        };

        let operator = if let Some(operator) = operator {
            let operator = operator.op.get_metadata_operator();
            quote::quote!(Some(#operator))
        } else {
            quote::quote!(None)
        };

        Ok(quote::quote! {
            Some(my_sqlite::SqlValueMetadata{
                sql_type: #sql_type,
                operator: #operator
            })
        })
    }

    fn fill_attributes(
        &self,
        fields: &mut Vec<TokenStream>,
        override_db_column_name: Option<DbColumnNameAttribute>,
    ) -> Result<(), syn::Error> {
        if let Some(override_db_column_name) = override_db_column_name {
            fields.push(override_db_column_name.generate_attribute());
        } else {
            if let Some(db_column_name) = self.get_db_column_name()?.attr {
                fields.push(db_column_name.generate_attribute());
            }
        }

        let sql_type_attribute: Option<SqlTypeAttribute> = self.try_get_attribute()?;

        if let Some(sql_type) = sql_type_attribute {
            fields.push(sql_type.generate_attribute());
        }

        Ok(())
    }

    fn get_sql_type_as_token_stream(&self) -> Result<proc_macro2::TokenStream, syn::Error> {
        let ty = if let PropertyType::OptionOf(ty) = &self.ty {
            ty.as_ref()
        } else {
            &self.ty
        };

        let ty_token = ty.get_token_stream_with_generics();

        let meta_data = self.get_field_metadata()?;

        Ok(quote::quote! {#ty_token:: get_sql_type(#meta_data)})
    }
}
