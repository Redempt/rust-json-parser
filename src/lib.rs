use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub enum JSONValue {

    Integer(i64),
    Decimal(f64),
    List(Vec<JSONValue>),
    Map(HashMap<String, JSONValue>),
    Boolean(bool),
    String(String),
    Null,

}

#[derive(Debug)]
pub struct ParseJSONError {

    error: String,

}

impl ParseJSONError {

    fn new(message: impl Into<String>) -> ParseJSONError {
        ParseJSONError {error: message.into()}
    }

}

impl FromStr for JSONValue {
    type Err = ParseJSONError;

    fn from_str(input: &str) -> Result<JSONValue, ParseJSONError> {
        let input = input.trim_start();
        let chars: Vec<char> = input.chars().collect();
        Ok(parse_json_trim(&chars)?.0)
    }

}

fn trim_start(input: &[char]) -> (&[char], usize) {
    let mut trimmed = 0;
    while trimmed < input.len() && input[trimmed].is_whitespace() {
        trimmed += 1;
    }
    (&input[trimmed..], trimmed)
}

fn parse_json_trim(input: &[char]) -> Result<(JSONValue, usize), ParseJSONError> {
    let (input, trimmed) = trim_start(input);
    let mut result = parse_json(input)?;
    result.1 += trimmed;
    Ok(result)
}

fn parse_json(input: &[char]) -> Result<(JSONValue, usize), ParseJSONError> {
    if input.len() == 0 {
        return Err(ParseJSONError::new("Empty input"));
    }
    return match input[0] {
        't' => Ok((JSONValue::Boolean(true), 4)),
        'f' => Ok((JSONValue::Boolean(false), 5)),
        'n' => Ok((JSONValue::Null, 4)),
        '"' => parse_string(input),
        '[' => parse_list(input),
        '{' => parse_map(input),
        '0'..='9' | '.' => parse_num(input),
        _ => Err(ParseJSONError::new("Invalid input"))
    }
}

fn parse_string(chars: &[char]) -> Result<(JSONValue, usize), ParseJSONError> {
    let mut out = String::new();
    let chars = &chars[1..];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' => {
                if i == chars.len() - 1 {
                    return Err(ParseJSONError::new(""));
                }
                let next = chars[i + 1];
                out.push(match next {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    _ => next
                });
                i += 1;
            }
            '"' => {
                return Ok((JSONValue::String(out), i + 2));
            }
            _ => out.push(c)
        }
        i += 1;
    }
    Err(ParseJSONError::new("Unclosed string"))
}

fn parse_list(chars: &[char]) -> Result<(JSONValue, usize), ParseJSONError> {
    let mut chars = &chars[1..];
    let mut list: Vec<JSONValue> = vec![];
    let mut length = 1;
    if chars[0] == ']' {
        return Ok((JSONValue::List(list), length));
    }
    loop {
        if chars.len() == 0 {
            return Err(ParseJSONError::new("Unclosed list"));
        }
        let next = parse_json_trim(chars)?;
        length += next.1;
        chars = &chars[next.1..];
        list.push(next.0);
        let (newchars, trimmed) = trim_start(chars);
        chars = newchars;
        length += trimmed + 1;
        if chars.len() == 0 {
            return Err(ParseJSONError::new("Unclosed list"));
        }
        match chars[0] {
            ',' => {
                chars = &chars[1..];
            }
            ']' => {
                return Ok((JSONValue::List(list), length))
            }
            _ => {
                return Err(ParseJSONError::new("Improperly delimited list"));
            }
        }
    }
}

fn parse_map(chars: &[char]) -> Result<(JSONValue, usize), ParseJSONError> {
    let mut chars = &chars[1..];
    let mut map: HashMap<String, JSONValue> = Default::default();
    let mut length = 1;
    if chars[0] == '}' {
        return Ok((JSONValue::Map(map), length));
    }
    loop {
        if chars.len() == 0 {
            return Err(ParseJSONError::new("Unclosed map"));
        }
        let (key, value, parsed) = parse_map_entry(chars)?;
        length += parsed;
        chars = &chars[parsed..];
        map.insert(key, value);
        let (newchars, trimmed) = trim_start(chars);
        chars = newchars;
        length += trimmed + 1;
        if chars.len() == 0 {
            return Err(ParseJSONError::new("Unclosed list"));
        }
        match chars[0] {
            ',' => {
                chars = &chars[1..];
            }
            '}' => {
                return Ok((JSONValue::Map(map), length))
            }
            _ => {
                return Err(ParseJSONError::new("Improperly delimited list"));
            }
        }
    }
}

fn parse_map_entry(chars: &[char]) -> Result<(String, JSONValue, usize), ParseJSONError> {
    let (key, mut length) = parse_json_trim(chars)?;
    let key = match key {
        JSONValue::String(s) => s,
        _ => return Err(ParseJSONError::new("Map key is not string"))
    };
    let chars = &chars[length..];
    let (mut chars, trimmed) = trim_start(chars);
    length += trimmed;
    if chars.len() == 0 || chars[0] != ':' {
        return Err(ParseJSONError::new("Improperly delimited map entry"));
    }
    length += 1;
    chars = &chars[1..];
    let (value, value_length) = parse_json_trim(chars)?;
    length += value_length;
    Ok((key, value, length))
}

fn parse_num(chars: &[char]) -> Result<(JSONValue, usize), ParseJSONError> {
    let mut num = 0i64;
    let negative = chars[0] == '-';
    let mut decimal_index = -1;
    let mut decimal = 0;
    let mut length = 0usize;
    let mut iter = chars.iter();
    if negative {
        iter.next();
        length += 1;
    }
    for c in iter {
        match c {
            '0'..='9' => {
                let to_inc = if decimal_index == -1 {&mut num} else {&mut decimal};
                *to_inc *= 10;
                *to_inc += (*c as i64) - ('0' as i64);
            }
            '.' => {
                if decimal_index != -1 {
                    return Err(ParseJSONError::new("Extra decimal point in number"));
                }
                decimal_index = length as i32;
            }
            _ => {
                return Ok((assemble_num(num, decimal_index, decimal, length, negative), length));
            }
        }
        length += 1;
    }
    Ok((assemble_num(num, decimal_index, decimal, length, negative), length))
}

fn assemble_num(num: i64, decimal_index: i32, decimal: i64, length: usize, negative: bool) -> JSONValue {
    let mut num = num;
    if negative {
        num = -num;
    }
    if decimal_index == -1 {
        return JSONValue::Integer(num);
    }
    let mut decimal = decimal as f64;
    let length = length as f64;
    let decimal_index = decimal_index as f64;
    decimal = 10f64.powf(-length + decimal_index + 1f64) * decimal;
    let mut num = num as f64 + decimal;
    if negative {
        num = -num;
    }
    JSONValue::Decimal(num)
}
