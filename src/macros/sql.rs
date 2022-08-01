use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use quote::ToTokens;
use regex::Regex;
use syn::{AttributeArgs, FnArg, ItemFn};

use crate::symbol;
use crate::util::{find_fn_body, find_return_type, get_page_req_ident, is_fetch, is_rbatis_ref};

pub(crate) fn impl_macro_sql(target_fn: &ItemFn, args: &AttributeArgs) -> TokenStream {
    let return_ty = find_return_type(target_fn);
    let func_name_ident = target_fn.sig.ident.to_token_stream();

    let mut rbatis_ident = "".to_token_stream();
    let mut rbatis_name = String::new();
    for x in &target_fn.sig.inputs {
        match x {
            FnArg::Receiver(_) => {}
            FnArg::Typed(t) => {
                let ty_stream = t.ty.to_token_stream().to_string();
                if is_rbatis_ref(&ty_stream) {
                    rbatis_ident = t.pat.to_token_stream();
                    rbatis_name = rbatis_ident
                        .to_string()
                        .trim_start_matches("mut ")
                        .to_string();
                    break;
                }
            }
        }
    }

    let sql_ident;
    if args.len() == 1 {
        if rbatis_name.is_empty() {
            panic!("[rbatis] you should add rbatis param  rb:&Rbatis  or rb: &mut RbatisExecutor<'_,'_>  on '{}()'!", target_fn.sig.ident);
        }
        sql_ident = args
            .get(0)
            .expect("[rbatis] missing sql macro param!")
            .to_token_stream();
    } else if args.len() == 2 {
        rbatis_ident = args
            .get(0)
            .expect("[rbatis] missing rbatis object identifier!")
            .to_token_stream();
        sql_ident = args
            .get(1)
            .expect("[rbatis] missing sql macro sql param!")
            .to_token_stream();
    } else {
        panic!("[rbatis] Incorrect macro parameter length!");
    }

    let func_args_stream = target_fn.sig.inputs.to_token_stream();
    let fn_body = find_fn_body(target_fn);
    let is_async = target_fn.sig.asyncness.is_some();
    if !is_async {
        panic!(
            "[rbaits] #[crud_table] 'fn {}({})' must be an async function! ",
            func_name_ident, func_args_stream
        );
    }
    if rbatis_ident.to_string().starts_with("mut ") {
        rbatis_ident = Ident::new(
            &rbatis_ident.to_string().trim_start_matches("mut "),
            Span::call_site(),
        )
        .to_token_stream();
    }
    let mut call_method;
    let is_fetch = is_fetch(&return_ty.to_string());
    if is_fetch {
        call_method = quote! {fetch};
    } else {
        call_method = quote! {exec};
    }
    //check use page method
    let mut page_req = quote! {};
    if return_ty.to_string().contains("Page <")
        && func_args_stream.to_string().contains("& PageRequest")
    {
        let req = get_page_req_ident(target_fn, &func_name_ident.to_string());
        page_req = quote! {,#req};
        call_method = quote! {fetch_page};
    }
    //append all args
    let sql_query = sql_ident.to_string();
    let sql_args_gen = generate_args(&sql_query);
    let sql_query: proc_macro2::TokenStream = convert_sql(&sql_query).parse().unwrap();
    //gen rust code templete
    let final_function_tokens = quote! {
       pub async fn #func_name_ident(#func_args_stream) -> #return_ty{
           let mut rb_args =vec![];
           #sql_args_gen
           #fn_body
           use rbatis::executor::{Executor,ExecutorMut};
           return #rbatis_ident.#call_method(&#sql_query, rb_args #page_req).await;
       }
    };
    return final_function_tokens.into();
}

fn generate_args(sql: &str) -> proc_macro2::TokenStream {
    let mut sql_args_gen = quote! {};
    let arg_regex = Regex::new(r"#\{(.*)\}").expect("Arg regex broken, please open github issue");
    for cap in arg_regex.captures_iter(sql) {
        let exp = &cap[1];
        let exp_token: proc_macro2::TokenStream = exp
            .parse()
            .expect(format!("Given expression could not be compiled: {}", exp).as_str());
        sql_args_gen = quote! {
            #sql_args_gen
            rb_args.push(rbson::to_bson(#exp_token).unwrap_or_default());
        };
    }
    sql_args_gen
}

fn convert_sql(sql: &str) -> String {
    let marker_replace_regex =
        Regex::new(r"#\{.*\}").expect("Marker regex broken, please open github issue");
    let total = marker_replace_regex.captures(sql).iter().count();
    let mut new_sql = sql.to_string();
    for i in 1..=total {
        new_sql = marker_replace_regex
            .replace(&new_sql, symbol(i))
            .to_string();
    }
    new_sql
}
