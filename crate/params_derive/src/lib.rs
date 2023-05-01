#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;

use syn::{
    Attribute, DeriveInput, Generics, Ident, ImplGenerics, Path, TypeGenerics, WhereClause,
    WherePredicate,
};

use crate::{
    fields_map::{fields_to_optional, fields_to_value_spec},
    impl_from_params_for_params_spec::impl_from_params_for_params_spec,
    impl_params_spec_for_params_spec::impl_params_spec_for_params_spec,
    impl_try_from_params_spec_for_params::impl_try_from_params_spec_for_params,
    type_gen::type_gen,
};

mod fields_map;
mod impl_from_params_for_params_spec;
mod impl_params_spec_for_params_spec;
mod impl_try_from_params_spec_for_params;
mod type_gen;
mod util;

/// Used to `#[derive]` the `Params` trait.
///
/// For regular usage, use `#[derive(Params)]`
///
/// For peace crates, also add the `#[peace_internal]` attribute, which
/// references the `peace_params` crate instead of the `peace::params`
/// re-export.
///
/// For types derived from `struct` `Param`s -- `Spec`, `Partial` -- we also:
///
/// * Generate getters and mut getters for non-`pub`, non-`PhantomData` fields.
/// * Generate a constructor if not all fields are `pub`.
///
/// Maybe we should also generate a `SpecBuilder` -- see commit `10f63611` which
/// removed builder generation.
#[proc_macro_derive(Params, attributes(peace_internal))]
pub fn data_access(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input)
        .expect("`Params` derive: Failed to parse item as struct, enum, or union.");

    let gen = impl_data_access(&ast);

    gen.into()
}

fn impl_data_access(ast: &DeriveInput) -> proc_macro2::TokenStream {
    let params_name = &ast.ident;

    let (peace_params_path, peace_resources_path): (Path, Path) = ast
        .attrs
        .iter()
        .find(peace_internal)
        .map(|_| (parse_quote!(peace_params), parse_quote!(peace_resources)))
        .unwrap_or_else(|| (parse_quote!(peace::params), parse_quote!(peace::resources)));

    let mut generics = ast.generics.clone();
    type_parameters_constrain(&mut generics);
    let generics_split = generics.split_for_impl();

    // MyParams -> MyParamsPartial
    let params_partial_name = {
        let mut params_partial_name = ast.ident.to_string();
        params_partial_name.push_str("Partial");
        Ident::new(&params_partial_name, ast.ident.span())
    };

    // MyParams -> MyParamsSpec
    let params_spec_name = {
        let mut params_spec_name = ast.ident.to_string();
        params_spec_name.push_str("Spec");
        Ident::new(&params_spec_name, ast.ident.span())
    };

    let params_partial = params_partial(ast, &generics_split, params_name, &params_partial_name);
    let params_spec = params_spec(
        ast,
        &generics_split,
        &peace_params_path,
        &peace_resources_path,
        params_name,
        &params_spec_name,
        &params_partial_name,
    );

    let (impl_generics, ty_generics, where_clause) = generics_split;

    quote! {
        impl #impl_generics #peace_params_path::Params
        for #params_name #ty_generics
        #where_clause
        {
            type Spec = #params_spec_name #ty_generics;
            type Partial = #params_partial_name #ty_generics;
        }

        #params_spec

        #params_partial
    }
}

fn peace_internal(attr: &&Attribute) -> bool {
    attr.path().is_ident("peace_internal")
}

/// Adds a `Send + Sync + 'static` bound on each of the type parameters.
fn type_parameters_constrain(generics: &mut Generics) {
    let generic_params = &generics.params;

    let where_predicates = generic_params
        .iter()
        .filter_map(|generic_param| match generic_param {
            syn::GenericParam::Lifetime(_) => None,
            syn::GenericParam::Type(type_param) => Some(type_param),
            syn::GenericParam::Const(_) => None,
        })
        .map(|type_param| parse_quote!(#type_param: Send + Sync + 'static))
        .collect::<Vec<WherePredicate>>();

    let where_clause = generics.make_where_clause();
    where_predicates
        .into_iter()
        .for_each(|where_predicate| where_clause.predicates.push(where_predicate));
}

/// Generates something like the following:
///
/// ```rust,ignore
/// #[derive(Clone, Debug, PartialEq, Eq)]
/// struct MyParamsPartial {
///     src: Option<PathBuf>,
///     dest_ip: Option<IpAddr>,
///     dest_path: Option<PathBuf>,
/// }
/// ```
fn params_partial(
    ast: &DeriveInput,
    generics_split: &(ImplGenerics, TypeGenerics, Option<&WhereClause>),
    params_name: &Ident,
    params_partial_name: &Ident,
) -> proc_macro2::TokenStream {
    let mut params_partial = type_gen(
        ast,
        generics_split,
        params_partial_name,
        fields_to_optional,
        &[
            parse_quote! {
                #[doc="\
                    Item spec parameters that may not necessarily have values.\n\
                    \n\
                    This is used for `try_state_current` and `try_state_desired` where values \n\
                    could be referenced from predecessors, which may not yet be available, such \n\
                    as the IP address of a server that is yet to be launched, or may change, \n\
                    such as the content hash of a file which is to be re-downloaded.\n\
                "]
            },
            parse_quote!(#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]),
        ],
    );

    params_partial.extend(impl_try_from_params_spec_for_params(
        ast,
        generics_split,
        params_name,
        params_partial_name,
    ));

    params_partial
}

/// Generates something like the following:
///
/// ```rust,ignore
/// struct MyParamsSpec {
///     src: peace_params::ValueSpec<PathBuf>,
///     dest_ip: peace_params::ValueSpec<IpAddr>,
///     dest_path: peace_params::ValueSpec<PathBuf>,
/// }
/// ```
fn params_spec(
    ast: &DeriveInput,
    generics_split: &(ImplGenerics, TypeGenerics, Option<&WhereClause>),
    peace_params_path: &Path,
    peace_resources_path: &Path,
    params_name: &Ident,
    params_spec_name: &Ident,
    params_partial_name: &Ident,
) -> proc_macro2::TokenStream {
    let mut params_spec = type_gen(
        ast,
        generics_split,
        params_spec_name,
        |fields| fields_to_value_spec(fields, peace_params_path),
        &[
            parse_quote! {
                #[doc="Specification of how to look up values for an item spec's parameters."]
            },
            // `Clone` and `Debug` are implemented manually, so that type parameters do not receive
            // the `Clone` and `Debug` bounds.
            parse_quote!(#[derive(serde::Serialize, serde::Deserialize)]),
        ],
    );

    params_spec.extend(impl_params_spec_for_params_spec(
        ast,
        generics_split,
        peace_params_path,
        peace_resources_path,
        params_name,
        params_spec_name,
        params_partial_name,
    ));

    params_spec.extend(impl_from_params_for_params_spec(
        ast,
        generics_split,
        peace_params_path,
        params_name,
        params_spec_name,
    ));

    params_spec
}
