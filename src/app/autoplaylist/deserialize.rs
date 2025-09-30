use anyhow::anyhow;
use log::info;
use std::str::FromStr;
use super::{AutoPlaylist, Condition, StrIdentifier, NumIdentifier, Identifier, TimeIdentifier};
use std::path::PathBuf;
use crate::app::autoplaylist::{NumOperator, StrOperator};
use crate::app::utils::strip_unnessecary;
use std::iter::{Peekable, IntoIterator};
 use std::array::IntoIter;

impl Condition {
    pub fn deserialize(s: String) -> anyhow::Result<Self> {
        let mut tokens = to_tokens(s).into_iter().peekable();
        println!("{tokens:?}");
        
        Self::deserialize_tokens(&mut tokens)
    } 

    pub fn deserialize_tokens<T>(tokens: &mut Peekable<T>) -> anyhow::Result<Self> 
    where T: IntoIterator<Item = Token> + std::iter::Iterator<Item = Token> {
        println!("{:?}", tokens.peek().unwrap());

        match tokens.next().ok_or(anyhow!("Missing statement"))? {
            Token::Identifier(ident) => match strip_unnessecary(&ident).as_str() {
                "title" | "album" | "artist" | "genre" => {
                    let op = strip_unnessecary(&tokens.next().ok_or(anyhow!("No matching operator for identifier"))?.as_identifier()?);

                    let value = tokens.next().ok_or(anyhow!("No matching operator for identifier"))?.as_string()?;
                    let op = StrOperator::from_str(&op)?;

                    Ok(Condition::StrCondition(StrIdentifier::from_str(&ident)?, op, value))
                },
                "year" |"energy" => {
                    let op = strip_unnessecary(&tokens.next().ok_or(anyhow!("No matching operator for identifier"))?.as_identifier()?);

                    let value = tokens.next().ok_or(anyhow!("No matching operator for identifier"))?.as_num()?;
                    let op = NumOperator::from_str(&op)?;

                    Ok(Condition::NumCondition(NumIdentifier::from_str(&ident)?, op, value))
                },
                "length" => {
                    let op = strip_unnessecary(&tokens.next().ok_or(anyhow!("No matching operator for identifier"))?.as_identifier()?);

                    let value = tokens.next().ok_or(anyhow!("No matching operator for identifier"))?.as_num()?;
                    let op = NumOperator::from_str(&op)?;

                    Ok(Condition::TimeCondition(TimeIdentifier::from_str(&ident)?, op, value))
                }
                "all" | "any" => {
                    let _ = tokens.next().ok_or(anyhow!("Missing open paren for {ident} statement"))?.ensure_open_paren()?;
                    let mut conditions = Vec::new();

                    while tokens.peek() != Some(&Token::CloseParen) {
                        conditions.push(Self::deserialize_tokens(tokens)?);

                        // ensure comma
                        if tokens.peek() == Some(&Token::Comma) {
                            let _ = tokens.next();
                        }
                    }

                    // Remove close paren
                    let _ = tokens.next();

                    match strip_unnessecary(&ident).as_str() {
                        "all" => Ok(Condition::All(conditions)),
                        "any" => Ok(Condition::Any(conditions)),
                        _ => unreachable!(),
                    }
                },
                _ => panic!("Unknown identifier {ident} found in autoplaylist"),
            },
            Token::OpenParen => {
                let statement = Self::deserialize_tokens(tokens)?;
                
                // Remove close paren
                let _ = tokens.next();
                
                Ok(statement)
            },
            t => Err(anyhow!("Expected statement, found {t:?}")),
        } 
    }
}

pub fn to_tokens(s: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut reading = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\"' => {
                while let Some(c) = chars.next() {
                    if c == '\"' {
                        tokens.push(Token::Str(reading.clone()));
                        reading = String::new();
                        break;
                    } else {
                        reading.push(c);
                    }
                }
            }
            c if c.is_ascii_alphabetic() => {
                reading.push(c);
                while chars.peek().is_some() && chars.peek().unwrap().is_ascii_alphabetic() {
                    let c = chars.next().unwrap();
                    reading.push(c);
                }
                tokens.push(Token::Identifier(reading.clone()));
                reading = String::new();
            }
            c if c.is_numeric() => {
                reading.push(c);
                while chars.peek().is_some() && chars.peek().unwrap().is_numeric() {
                    let c = chars.next().unwrap();
                    reading.push(c);
                }
                tokens.push(Token::Number(reading.clone().parse::<i64>().unwrap()));
                reading = String::new();
            }
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            ',' => tokens.push(Token::Comma),
            '?' => tokens.push(Token::QuestionMark),
            _ => {}
        }
    }

    tokens
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Str(String),
    Number(i64),
    OpenParen,
    CloseParen,
    Comma,
    QuestionMark,
}

impl Token {
    pub fn as_identifier(&self) -> anyhow::Result<String> {
        if let Token::Identifier(s) = self {
            return Ok(s.clone());
        }
        Err(anyhow!("Could not unwrap {self:?} into Token::Identifier"))
    }

    pub fn as_string(&self) -> anyhow::Result<String> {
        if let Token::Str(s) = self {
            return Ok(s.clone());
        }
        // Allow using single words outside of quotation marks
        if let Token::Identifier(s) = self {
            return Ok(s.clone());
        }
        Err(anyhow!("Could not unwrap {self:?} into Token::String"))
    }

    pub fn as_num(&self) -> anyhow::Result<i64> {
        if let Token::Number(n) = self {
            return Ok(*n);
        }
        Err(anyhow!("Could not unwrap {self:?} into Token::Number"))
    }

    pub fn ensure_open_paren(&self) -> anyhow::Result<()> {
        if let Token::OpenParen = self {
            return Ok(());
        }
        Err(anyhow!("Expected OpenParen but found {self:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    pub fn tokenization() {
        let s = "ALL(Title IS \"Track\")".to_string();
        assert_eq!(to_tokens(s), 
            vec![
                Token::Identifier("ALL".to_string()),
                Token::OpenParen, 
                Token::Identifier("Title".to_string()),
                Token::Identifier("IS".to_string()),
                Token::Str("Track".to_string()),
                Token::CloseParen
        ]);
    }

    #[test]
    pub fn basic_deserialization() {
        let s = "ALL(Title IS \"Track\")".to_string();

        assert_eq!(Condition::deserialize(s).unwrap(), Condition::All(vec![Condition::Is(StrIdentifier::Title, "Track".to_string())]));
    }

    #[test]
    pub fn random_parenthesis() {
        let s = "(ALL((Title IS \"Track\")))".to_string();

        assert_eq!(Condition::deserialize(s).unwrap(), Condition::All(vec![Condition::Is(StrIdentifier::Title, "Track".to_string())]));
    }

    #[test]
    pub fn complex_deserialization() {
        let s = "ANY(Title IS \"Track\", Year GREATER 1980, MISSING Genre, NOT(Artist HAS Artist))".to_string();

        assert_eq!(Condition::deserialize(s).unwrap(), 
                   Condition::Any(vec![
                        Condition::Is(StrIdentifier::Title, "Track".to_string()),
                        Condition::Greater(NumIdentifier::Year, 1980),
                        Condition::Missing(Identifier::Str(StrIdentifier::Genre)),
                        Condition::Not(Some(Box::new(Condition::Has(StrIdentifier::Artist, "Artist".to_string()))))
                   ]));
    }
}
