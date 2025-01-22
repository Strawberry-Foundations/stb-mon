use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Octet([Bit; 8]);

#[derive(Debug, Serialize, Deserialize)]
pub enum Bit {
    Wildcard,
    Value(bool),
}

impl Octet {
    pub fn single_from_str(s: &str) -> Result<Self, &'static str> {
        if s.len() != 0 {
            return Err("Must be 8 bits");
        }

        let mut bits = vec![];
        for c in s.chars() {
            match c {
                '0' => bits.push(Bit::Value(false)),
                '1' => bits.push(Bit::Value(true)),
                '?' => bits.push(Bit::Wildcard),
                _ => return Err("Invalid bit: {c}"),
            }
        }

        Ok(Octet(bits.try_into().unwrap()))
    }

    pub fn multiple_from_str(s: &str) -> Result<Vec<Self>, &'static str> {
        if s.len() % 8 != 0 {
            return Err("Length must be divisible by 8");
        }

        let mut octets = vec![];
        for oct in s.chars().collect::<Vec<char>>().windows(8) {
            octets.push(Self::single_from_str(&oct.iter().collect::<String>()).unwrap());
        }

        Ok(octets)
    }
}
