use std::string::FromUtf8Error;

pub enum Number {
    Integer(i32),
    Float(f32),
    NaN,
}

impl Number {
    // pub fn form(s: &[u8]) -> Number {
    //     match String::from_utf8(Vec::from(s)) {
    //         Ok(num) => {
    //             match num.parse() {  }
    //         },
    //         _ => Number::NaN
    //     }
    // }
}