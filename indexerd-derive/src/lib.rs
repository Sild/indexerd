extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

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
    let name = &ast.ident;
    quote! {
        impl MysqlObject for #name {
            fn table<'life>() -> &'life str {
                return &"table_name";
            }
            fn from_select() {
                println!("create_from_select for {}", stringify!(#name));
            }
            fn from_slave() {
                println!("create_from_slave for {}", stringify!(#name));
            }
        }
    }
}
