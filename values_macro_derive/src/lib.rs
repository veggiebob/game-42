use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};
use serde::{Deserialize, Serialize};

#[proc_macro_derive(EnumValues)]
pub fn values_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    if let Data::Enum(enum_data) = &ast.data {
        let mut variants = vec![];
        for v in enum_data.variants.iter() {
            if let Fields::Unit = v.fields {
                let var_name = &v.ident;
                variants.push(quote! { #name::#var_name });
            } else {
                panic!(
                    "Values macro can only be applied to \
                        enums with Unit variants (no fields). '{}' is not a Unit variant",
                    v.ident
                );
            }
        }
        let variant_count = variants.len();
        let values_impl = quote! {
            impl #name {

                /// Produces an iterator over owned variants of this enum.
                pub fn values() -> impl Iterator<Item=#name> {
                    vec![
                        #(#variants),*
                    ].into_iter()
                }

                /// Gives the number of variants in this enum.
                pub fn len() -> usize {
                    #variant_count
                }
            }
        };
        values_impl.into()
    } else {
        panic!("Values macro can only be applied to enums.");
    }
}

#[proc_macro_derive(Mapping)]
pub fn mapping_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let visibility = &ast.vis;
    if let Data::Enum(enum_data) = &ast.data {
        let mut variants = vec![];
        for v in enum_data.variants.iter() {
            if let Fields::Unit = v.fields {
                let var_name = &v.ident;
                variants.push(quote! { #name::#var_name });
            } else {
                panic!(
                    "Mapping macro can only be applied to \
                        enums with Unit variants (no fields). '{}' is not a Unit variant",
                    v.ident
                );
            }
        }
        let variant_count = variants.len();
        let map_name = format_ident!("{}Mapping", name);
        let into_iter_name = format_ident!("{}MappingIntoIter", name);
        let iter_name = format_ident!("{}MappingIter", name);
        let cases: Vec<_> = variants
            .iter()
            .enumerate()
            .map(|(i, var)| {
                quote! { #var => #i }
            })
            .collect();

        //
        let puts_construct: Vec<_> = variants
            .iter()
            .enumerate()
            .map(|(_, var)| {
                quote! { f(#var) }
            })
            .collect();

        // cases of a match that map from enum index to enum value
        let rcases: Vec<_> = variants
            .iter()
            .enumerate()
            .map(|(i, var)| {
                if i == variant_count - 1 {
                    return quote! { _ => #var };
                } else {
                    quote! { #i => #var }
                }
            })
            .collect();

        let values_impl = quote! {
            #[derive(Copy, Clone, Serialize, Deserialize)]
            #visibility struct #map_name <T>([T; #variant_count]);
            impl<T> #map_name<T> {
                #visibility fn get(&self, var: #name) -> &T {
                    let index = match var {
                        #(#cases),*
                    };
                    &self.0[index]
                }
                #visibility fn get_mut(&mut self, var: #name) -> &mut T {
                    let index = match var {
                        #(#cases),*
                    };
                    &mut self.0[index]
                }
                #visibility fn put(&mut self, var: #name, val: T) {
                    let index = match var {
                        #(#cases),*
                    };
                    self.0[index] = val;
                }
                #visibility fn new<F: FnMut(#name) -> T>(mut f: F) -> Self {
                    let arr = [#(#puts_construct),*,];
                    #map_name(arr)
                }
                #visibility fn iter(&self) -> #iter_name<T> {
                    self.into_iter()
                }
                #visibility fn into_iter(self) -> #into_iter_name<T> {
                    self.into_iter()
                }
            }


            #visibility struct #into_iter_name<T>(Vec<T>, usize);
            #visibility struct #iter_name<'a, T>(&'a #map_name<T>, usize);
            impl<T> IntoIterator for #map_name<T> {
                type Item = (#name, T);
                type IntoIter = #into_iter_name<T>;

                fn into_iter(self) -> Self::IntoIter {
                    #into_iter_name(self.0.into_iter().rev().collect(), 0)
                }
            }

            impl<'a, T> IntoIterator for &'a #map_name<T> {
                type Item = (#name, &'a T);
                type IntoIter = #iter_name<'a, T>;

                fn into_iter(self) -> Self::IntoIter {
                    #iter_name(self, 0)
                }
            }

            impl<T> Iterator for #into_iter_name<T> {
                type Item = (#name, T);

                fn next(&mut self) -> Option<Self::Item> {
                    self.0.pop().map(|t| {
                        let i = self.1;
                        self.1 += 1;
                        (
                            match i {
                                #(#rcases),*
                            },
                            t
                        )
                    })
                }
            }

            impl<'a, T> Iterator for #iter_name<'a, T> {
                type Item = (#name, &'a T);

                fn next(&mut self) -> Option<Self::Item> {
                    if self.1 < #variant_count {
                        let i = self.1;
                        self.1 += 1;
                        Some((
                            match i {
                                #(#rcases),*
                            },
                            &self.0.0[i]
                        ))
                    } else {
                        None
                    }
                }
            }

        };
        values_impl.into()
    } else {
        panic!("Mapping macro can only be applied to enums.");
    }
}


#[cfg(test)]
mod test {
    #[test]
    pub fn test() {
        let mapping = [1, 2, 3, 4];
        let mut v: Vec<i32> = mapping.into_iter().collect();
        if let Some(x) = v.pop() {}
    }
}