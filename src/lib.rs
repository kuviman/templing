extern crate proc_macro;

use std::fmt::Write;

#[proc_macro]
pub fn templing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::LitStr = syn::parse(input).expect("Expected a string literal as input");
    templing_impl(input.value().as_str(), Vec::new())
        .parse()
        .unwrap()
}

#[proc_macro]
pub fn dbg_templing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::LitStr = syn::parse(input).expect("Expected a string literal as input");
    panic!("{}", templing_impl(input.value().as_str(), Vec::new()));
}

#[proc_macro]
pub fn include_templing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::LitStr = syn::parse(input).expect("Expected a string literal as input");
    let cargo_manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = std::path::Path::new(&cargo_manifest_dir).join(input.value());
    let input = std::fs::read_to_string(&path).expect(&format!("Failed to read {:?}", path));
    templing_impl(input.as_str(), vec![path]).parse().unwrap()
}

#[proc_macro]
pub fn dbg_include_templing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::LitStr = syn::parse(input).expect("Expected a string literal as input");
    let cargo_manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = std::path::Path::new(&cargo_manifest_dir).join(input.value());
    let input = std::fs::read_to_string(&path).expect(&format!("Failed to read {:?}", path));
    panic!("{}", templing_impl(input.as_str(), vec![path]));
}

fn templing_impl(input: &str, file_dependencies: Vec<std::path::PathBuf>) -> String {
    let mut result = String::new();
    writeln!(&mut result, "{{").unwrap();
    for file in file_dependencies {
        writeln!(&mut result, "include_bytes!({:?});", file).unwrap();
    }
    writeln!(&mut result, "let mut result = String::new();").unwrap();
    for line in input.lines() {
        let non_ws = line.trim();
        if let Some(code) = non_ws.strip_prefix("- ") {
            writeln!(&mut result, "{}", code).unwrap();
        } else {
            let mut line = line;
            while !line.trim().is_empty() {
                let index = match line.find("{{") {
                    Some(index) => index,
                    None => line.len(),
                };
                writeln!(&mut result, "result.push_str({:?});", &line[..index]).unwrap();
                if index < line.len() {
                    line = &line[index + 2..];
                    let index = line.find("}}").expect("Failed to find closing brackets");
                    let code = &line[..index];
                    if code.chars().next() == Some('#') {
                        writeln!(&mut result, "{}", &code[1..]).unwrap();
                    } else {
                        writeln!(&mut result, "result.push_str(&{{ {} }}.to_string());", code)
                            .unwrap();
                    }
                    line = &line[index + 2..];
                } else {
                    line = "";
                }
            }
            writeln!(&mut result, "result.push('\\n');").unwrap();
        }
    }
    writeln!(&mut result, "result").unwrap();
    writeln!(&mut result, "}}").unwrap();
    result
}
