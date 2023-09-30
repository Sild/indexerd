extern crate inflections;
extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;
use inflections::case::to_snake_case;
use proc_macro::TokenStream;
use syn::Ident;
use syn::Ty;
use syn::VariantData;

#[proc_macro_derive(MysqlObject)]
pub fn mysql_object(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();
    // Build the impl
    let gen = mysql_object_impl(&ast);
    // Return the generated impl
    gen.parse().unwrap()
}
struct FieldInfo {
    pub name: Option<Ident>,
    pub t: Ty,
}

fn mysql_object_impl(ast: &syn::DeriveInput) -> quote::Tokens {
    let class_name = &ast.ident;
    let table_name = to_snake_case(&class_name.to_string());

    let field_name_type = match ast.body {
        syn::Body::Struct(ref data) => match data {
            VariantData::Struct(ref fields) => fields
                .iter()
                .map(|x| FieldInfo {
                    name: x.ident.clone(),
                    t: x.ty.clone(),
                })
                .collect::<Vec<_>>(),
            _ => panic!("MysqlObject can only be derived for structs"),
        },
        _ => panic!("MysqlObject can only be derived for structs"),
    };

    let member_init = field_name_type
        .iter()
        .map(|x| {
            let (name, ty) = (x.name.clone(), &x.t);
            quote!(#name: convert::<#ty>(row_data.cells.get(mapping.get(stringify!(#name)).unwrap().clone()).unwrap()),)
        })
        .collect::<Vec<_>>();

    quote! {
        impl MysqlObject for #class_name {

            fn table<'life>() -> &'life str
            where Self: Sized {
                &#table_name
            }

            fn from_slave(row_data: &mysql_cdc::events::row_events::row_data::RowData, fields_map: &HashMap<String, FieldMapping>) -> Self
            where Self: Sized {
                // #class_name::default()
                let mapping = fields_map.get(Self::table()).expect("No mapping found for table");

                // println!("create_from_slave for {}", stringify!(#class_name));
                // println!("fields: {:?}", stringify!(#field_names));
                let mut obj = #class_name {
                    #(#member_init)*
                };
                obj
            }
        }
    }
}
