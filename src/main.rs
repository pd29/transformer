use clap::{crate_authors, crate_version, App, Arg};
use glob::glob;
use serde_json::Value;
use std::{collections::HashMap, error::Error, path::Path};

fn main() {
    let matches = App::new("transformer")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Transforms .net core settings json to docker env files")
        .args(&[
            Arg::with_name("pattern").help("Glob pattern"),
            Arg::with_name("variables").help("Variable file for replacement with format: %VAR_NAME%=VALUE"),
        ])
        .get_matches();

    traverse(matches.value_of("pattern"), matches.value_of("variables")).unwrap()
}

fn traverse<P: AsRef<Path>>(pattern: Option<&str>, vars_file: Option<P>) -> Result<(), Box<dyn Error>> {
    let vars_file_content: String;
    let vars = if let Some(vars_file) = vars_file {
        vars_file_content = std::fs::read_to_string(vars_file)?;
        vars_file_content
            .lines()
            .map(|l| {
                let mut ls = l.split_whitespace();
                match (ls.next(), ls.next()) {
                    (Some(a), Some(b)) => Ok((a, b)),
                    _ => Err("wrong file format"),
                }
            })
            .filter_map(Result::ok)
            .collect()
    } else {
        HashMap::new()
    };

    for entry in glob(pattern.unwrap_or("**/*.json"))?.filter_map(Result::ok) {
        println!("// {:?}", entry);
        read_json_from_file(entry).ok().map(|v| transform(&v, &[], &vars));
    }
    Ok(())
}

fn transform(value: &Value, path: &[&str], vars: &HashMap<&str, &str>) {
    let prefix = path.join("__");
    match value {
        Value::Object(o) => o.iter().for_each(|(n, v)| transform(v, &append(path, n), vars)),
        Value::Array(a) => a
            .iter()
            .enumerate()
            .for_each(|(i, v)| transform(v, &append(path, &i.to_string()), &vars)),
        _ => println!("{} = {}", prefix, try_substitute(value)),
    }
}

fn try_substitute(value: &Value) -> String {
    value.to_string()
}

fn read_json_from_file<P: AsRef<Path>>(path: P) -> Result<Value, Box<dyn Error>> {
    let buf = std::fs::read(path)?;
    let v = serde_json::from_slice(strip_bom(&buf))?;
    Ok(v)
}

fn strip_bom(buf: &[u8]) -> &[u8] {
    let b = if is_bom(&buf) { &buf[3..] } else { &buf[..] };
    b
}

fn is_bom(buf: &[u8]) -> bool {
    buf.len() > 2 && buf[0] == 0xef && buf[1] == 0xbb && buf[2] == 0xbf
}

fn append<'a>(a: &[&'a str], b: &'a str) -> Vec<&'a str> {
    [a, &[b]].concat()
}
