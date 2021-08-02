use std::collections::{HashMap, LinkedList};
use parser::Parser;

#[derive(Debug)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub fn parse(code: &str) -> Result<HashMap<String, JsonValue>, ()> {
    Ok(JsonParser::new(code).parse().unwrap())
}

struct JsonParser<'a> {
    parser: Parser<'a>,
}

impl<'a> JsonParser<'a> {
    fn new(code: &'a str) -> Self {
        JsonParser {
            parser: Parser::from(code),
        }
    }

    fn parse(&mut self) -> Result<HashMap<String, JsonValue>, ()> {
        self.parser.skip();
        if self.parser.next() != '{' {
            return Err(());
        }
        
        self.parser.skip();
        if self.parser.get_char() == '}' {
            self.parser.next();
            self.parser.skip();

            if !self.parser.finished() {
                return Err(());
            }

            return Ok(HashMap::new());
        }

        let mut stack = LinkedList::new();
        stack.push_back((JsonValue::Object(HashMap::new()), None));

        while !self.parser.finished() && !stack.is_empty() {
            match &mut stack.back_mut().unwrap().0 {
                JsonValue::Array(array) => {
                    self.parser.skip();
                    match self.parser.get_char() {
                        '[' => {
                            self.parser.next();
                            stack.push_back((JsonValue::Array(Vec::new()), None));
                            continue;
                        }
                        '{' => {
                            self.parser.next();
                            stack.push_back((JsonValue::Object(HashMap::new()), None));
                            continue;
                        }
                        _ => {
                            array.push(self.get_value()?);
                        }
                    }
                }
                JsonValue::Object(object) => {
                    self.parser.skip();
                    let key = self.get_key()?;
            
                    self.parser.skip();
                    if self.parser.next() != ':' {
                        return Err(());
                    }

                    self.parser.skip();
                    match self.parser.get_char() {
                        '[' => {
                            self.parser.next();
                            stack.push_back((JsonValue::Array(Vec::new()), Some(key)));
                            continue;
                        }
                        '{' => {
                            self.parser.next();
                            stack.push_back((JsonValue::Object(HashMap::new()), Some(key)));
                            continue;
                        }
                        _ => {
                            object.insert(key, self.get_value()?);
                        }
                    }
                }
                _ => unreachable!(),
            }

            while !self.parser.finished() {
                self.parser.skip();
                match self.parser.next() {
                    ',' => break,
                    ']' => {
                        let last = stack.pop_back().unwrap();
                        if stack.is_empty() {
                            match last.0 {
                                JsonValue::Object(val) => return Ok(val),
                                _ => unreachable!(),
                            };
                        }
    
                        match last.0 {
                            JsonValue::Array(array) => {
                                match &mut stack.back_mut().unwrap().0 {
                                    JsonValue::Array(val) => {
                                        val.push(JsonValue::Array(array));
                                    }
                                    JsonValue::Object(val) => {
                                        val.insert(last.1.unwrap(), JsonValue::Array(array));
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            JsonValue::Object(_) => return Err(()),
                            _ => unreachable!(),
                        }
                    }
                    '}' => {
                        let last = stack.pop_back().unwrap();
                        if stack.is_empty() {
                            match last.0 {
                                JsonValue::Object(val) => return Ok(val),
                                _ => unreachable!(),
                            };
                        }
    
                        match last.0 {
                            JsonValue::Array(_) => return Err(()),
                            JsonValue::Object(object) => {
                                match &mut stack.back_mut().unwrap().0 {
                                    JsonValue::Array(val) => {
                                        val.push(JsonValue::Object(object));
                                    }
                                    JsonValue::Object(val) => {
                                        val.insert(last.1.unwrap(), JsonValue::Object(object));
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => return Err(()),
                }
            }
        }

        unreachable!()
    }

    fn get_key(&self) -> Result<String, ()> {
        if self.parser.next() != '"' {
            return Err(());
        }

        let key = self.parser.get_to_char('"').to_string();

        if self.parser.next() != '"' {
            return Err(());
        }

        Ok(key)
    }

    fn get_value(&self) -> Result<JsonValue, ()> {
        if self.parser.get_char() == '"' {
            self.parser.next();
            return self.get_str_value();
        }
        
        self.get_number_or_boolean_value()
    }
    
    fn get_str_value(&self) -> Result<JsonValue, ()> {
        let string = self.parser.get_to_char('"').to_string();

        if self.parser.next() != '"' {
            return Err(());
        }
        
        Ok(JsonValue::String(string))
    }
    
    fn get_number_or_boolean_value(&self) -> Result<JsonValue, ()> {
        let break_chars = [' ', '\n', '\t', '\r', ',', ']', '}'];
    
        if !self.parser.get_char().is_ascii_digit() {
            let word = self.parser.get_to_chars(&break_chars);
            
            return match word {
                "null" => Ok(JsonValue::Null),
                "false" => Ok(JsonValue::Boolean(false)),
                "true" => Ok(JsonValue::Boolean(true)),
                _ => Err(()),
            };
        }
        
        let number = self.parser.get_to_chars(&break_chars);
        let number = match number.parse::<f64>() {
            Ok(val) => val,
            Err(_) => return Err(()),
        };
    
        Ok(JsonValue::Number(number))
    }
}

enum JsonIter<'a> {
    Array(std::slice::Iter<'a, JsonValue>),
    Object(std::collections::hash_map::Iter<'a, String, JsonValue>),
}

pub fn generate(json: &HashMap<String, JsonValue>) -> String {
    let mut buffer = String::from("{");

    let mut stack = LinkedList::new();
    stack.push_back((true, JsonIter::Object(json.iter())));

    'mainLoop: while let Some((mut first_time, iterator)) = stack.pop_back() {
        match iterator {
            JsonIter::Array(mut array_iter) => {
                while let Some(value) = array_iter.next() {
                    if first_time {
                        first_time = false;
                    }
                    else {
                        buffer.push_str(", ");
                    }

                    match value {
                        JsonValue::Null => buffer.push_str("null"),
                        JsonValue::Boolean(val) => buffer.push_str(&val.to_string()),
                        JsonValue::Number(val) => buffer.push_str(&val.to_string()),
                        JsonValue::String(val) => buffer.push_str(&format!(r#""{}""#, val)),
                        JsonValue::Array(val) => {
                            buffer.push('[');

                            stack.push_back((first_time, JsonIter::Array(array_iter)));
                            stack.push_back((true, JsonIter::Array(val.iter())));
                            continue 'mainLoop;
                        }
                        JsonValue::Object(val) => {
                            buffer.push('{');

                            stack.push_back((first_time, JsonIter::Array(array_iter)));
                            stack.push_back((true, JsonIter::Object(val.iter())));
                            continue 'mainLoop;
                        }
                    }
                }

                buffer.push(']');
            }
            JsonIter::Object(mut object_iter) => {
                while let Some(value) = object_iter.next() {
                    if first_time {
                        first_time = false;
                    }
                    else {
                        buffer.push_str(", ");
                    }

                    match value.1 {
                        JsonValue::Null => buffer.push_str(&format!(r#""{}": null"#, value.0)),
                        JsonValue::Boolean(val) => buffer.push_str(&format!(r#""{}": {}"#, value.0, val)),
                        JsonValue::Number(val) => buffer.push_str(&format!(r#""{}": {}"#, value.0, val)),
                        JsonValue::String(val) => buffer.push_str(&format!(r#""{}": "{}""#, value.0, val)),
                        JsonValue::Array(val) => {
                            buffer.push_str(&format!(r#""{}": ["#, value.0));

                            stack.push_back((first_time, JsonIter::Object(object_iter)));
                            stack.push_back((true, JsonIter::Array(val.iter())));
                            continue 'mainLoop;
                        }
                        JsonValue::Object(val) => {
                            buffer.push_str(&format!(r#""{}": {{"#, value.0));

                            stack.push_back((first_time, JsonIter::Object(object_iter)));
                            stack.push_back((true, JsonIter::Object(val.iter())));
                            continue 'mainLoop;
                        }
                    }
                }

                buffer.push('}');
            }
        }
    }

    buffer
}