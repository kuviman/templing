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
    writeln!(&mut result, "let mut templing_result = String::new();").unwrap();
    let mut current_line = 0;
    for line in input.lines() {
        current_line += 1;
        let non_ws = line.trim();
        if let Some(code) = non_ws.strip_prefix("- ") {
            writeln!(&mut result, "{}", code).unwrap();
        } else {
            let mut current_column = 1;
            let mut line = line.to_owned();
            if line.trim().starts_with('\\') {
                line = line.replacen('\\', "", 1);
                current_column += 1;
            }
            let mut line = line.as_str();
            let mut write_eol = true;
            if line.trim().starts_with('~') {
                let index = line.find('~').unwrap();
                current_column += line[index + 1..].chars().count();
                line = &line[index + 1..];
            }
            if line.trim().ends_with('~') {
                let index = line.rfind('~').unwrap();
                line = &line[..index];
                write_eol = false;
            }
            while !line.is_empty() {
                let index = match line.find("{{") {
                    Some(index) => index,
                    None => line.len(),
                };
                writeln!(
                    &mut result,
                    "templing_result.push_str({:?});",
                    &line[..index],
                )
                .unwrap();
                if index < line.len() {
                    current_column += line[..index + 2].chars().count();
                    writeln!(&mut result, "let templing_indentation = (templing_result.len() - templing_result.rfind('\\n').map(|pos| pos + 1).unwrap_or(0));").unwrap();
                    line = &line[index + 2..];
                    let index = line.find("}}").expect(&format!(
                        "Failed to find closing brackets for {}:{}",
                        current_line,
                        current_column - 2,
                    ));
                    let code = &line[..index];
                    if code.chars().next() == Some('#') {
                        writeln!(&mut result, "{}", &code[1..]).unwrap();
                    } else {
                        writeln!(
                            &mut result,
                            "let templing_value = {{ {} }}.to_string();",
                            code,
                        )
                        .unwrap();
                        writeln!(&mut result, "let templing_value = templing_value.trim();")
                            .unwrap();
                        writeln!(&mut result, "for (templing_part_index, templing_part) in templing_value.split('\\n').enumerate() {{").unwrap();
                        writeln!(&mut result, "if templing_part_index != 0 {{").unwrap();
                        writeln!(&mut result, "templing_result.push('\\n');").unwrap();
                        writeln!(&mut result, "for _ in 0..templing_indentation {{").unwrap();
                        writeln!(&mut result, "templing_result.push(' ');").unwrap();
                        writeln!(&mut result, "}}").unwrap();
                        writeln!(&mut result, "}}").unwrap();
                        writeln!(&mut result, "templing_result.push_str(templing_part);").unwrap();
                        writeln!(&mut result, "}}").unwrap();
                    }
                    current_column += line[..index + 2].chars().count();
                    line = &line[index + 2..];
                } else {
                    line = "";
                }
            }
            if write_eol {
                writeln!(&mut result, "templing_result.push('\\n');").unwrap();
            }
        }
    }
    writeln!(&mut result, "templing_result.trim().to_owned()").unwrap();
    writeln!(&mut result, "}}").unwrap();
    result
}
