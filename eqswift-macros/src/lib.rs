use proc_macro::TokenStream;
use quote::quote;

/// Export a function, method, impl block, or trait to foreign languages.
///
/// Thin wrapper around `#[uniffi::export]`:
///
/// ```ignore
/// #[eqswift::export]
/// fn hello(name: String) -> String {
///     format!("Hello, {name}!")
/// }
/// ```
///
/// For constructors inside exported impl blocks, use `#[uniffi::constructor]`:
///
/// ```ignore
/// #[derive(eqswift::Object)]
/// pub struct Greeter;
///
/// #[eqswift::export]
/// impl Greeter {
///     #[uniffi::constructor]
///     pub fn new() -> Self { Self }
///
///     pub fn greet(&self, name: String) -> String {
///         format!("Hello, {name}!")
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn export(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = proc_macro2::TokenStream::from(args);
    let input = proc_macro2::TokenStream::from(input);

    quote! {
        #[::eqswift::__uniffi_export(#args)]
        #input
    }
    .into()
}

/// One-time setup macro.
///
/// Expands to `uniffi::setup_scaffolding!()`. Call **once** at the top of your
/// crate root (`lib.rs`) before any exported items.
///
/// ```ignore
/// eqswift::setup!();
///
/// #[eqswift::export]
/// pub fn add(a: u32, b: u32) -> u32 { a + b }
/// ```
#[proc_macro]
pub fn setup(_input: TokenStream) -> TokenStream {
    quote! {
        ::eqswift::setup_scaffolding!();
    }
    .into()
}
