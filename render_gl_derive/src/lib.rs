#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = syn::parse_macro_input!(_input as syn::DeriveInput);

    // Build the output, possibly using quasi-quotation
    let expanded = generate_impl(&ast);

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

fn generate_impl(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields = match &ast.data {
        syn::Data::Struct(data) => &data.fields,
        _ => panic!("Only Structs are supported"),
    };
    let mut generated_code = Vec::new();
    for field in fields {
        let field_name = &field.ident.clone().unwrap();
        let field_ty = &field.ty;
        let location = match get_location(&field.attrs) {
            Some(location) => location,
            None => {
                return quote! {
                        compile_error!(concat!("Missing or invalid location attribute for field ", stringify!(#field_name)));
                }
            }
        };
        generated_code.push(quote! {
            unsafe {
                #field_ty::vertex_attrib_pointer(gl, stride, #location, offset);
            }
            let offset = offset + std::mem::size_of::<Self>();
        });
    }
    quote! {
        impl #ident #generics #where_clause {
            #[allow(unused_variables)]
            pub fn vertex_attrib_pointers(gl: &::gl::Gl) {
                let stride = ::std::mem::size_of::<Self>();
                let offset = 0;
                #(#generated_code)*
            }
        }
    }
}

fn get_location(attrs: &[syn::Attribute]) -> Option<proc_macro2::TokenStream> {
    attrs
        .iter()
        .find_map(|attr| match attr.parse_args::<syn::Expr>() {
            Ok(expr) => Some(quote! { #expr }),
            Err(e) => {
                panic!("Oops! Could not get the location value: {}", e)
            }
        })
}
