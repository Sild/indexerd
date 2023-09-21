extern crate inflections;
extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;
use inflections::case::to_snake_case;
use proc_macro::TokenStream;

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

fn mysql_object_impl(ast: &syn::DeriveInput) -> quote::Tokens {
    let class_name = &ast.ident;
    let table_name = to_snake_case(&class_name.to_string());

    quote! {
        impl MysqlObject for #class_name {
            fn table<'life>() -> &'life str {
                &#table_name
            }
            fn from_slave() -> Self {
                println!("create_from_slave for {}", stringify!(#class_name));
                #class_name::default()
            }
        }
    }
}
