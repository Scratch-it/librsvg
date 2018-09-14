use std::error;
use std::fmt;

use cairo;
use cssparser::BasicParseError;

use attributes::Attribute;
use parsers::ParseError;

/// A simple error which refers to an attribute's value
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeError {
    /// The value could not be parsed
    Parse(ParseError),

    // The value could be parsed, but is invalid
    Value(String),
}

/// A complete error for an attribute and its erroneous value
#[derive(Debug, Clone, PartialEq)]
pub struct NodeError {
    attr: Attribute,
    err: AttributeError,
}

impl NodeError {
    pub fn parse_error(attr: Attribute, error: ParseError) -> NodeError {
        NodeError {
            attr,
            err: AttributeError::Parse(error),
        }
    }

    pub fn value_error(attr: Attribute, description: &str) -> NodeError {
        NodeError {
            attr,
            err: AttributeError::Value(description.to_string()),
        }
    }

    pub fn attribute_error(attr: Attribute, error: AttributeError) -> NodeError {
        NodeError { attr, err: error }
    }
}

impl error::Error for NodeError {
    fn description(&self) -> &str {
        match self.err {
            AttributeError::Parse(_) => "parse error",
            AttributeError::Value(_) => "invalid attribute value",
        }
    }
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.err {
            AttributeError::Parse(ref n) => write!(
                f,
                "error parsing value for attribute \"{}\": {}",
                self.attr.to_string(),
                n.display
            ),

            AttributeError::Value(ref s) => write!(
                f,
                "invalid value for attribute \"{}\": {}",
                self.attr.to_string(),
                s
            ),
        }
    }
}

impl From<ParseError> for AttributeError {
    fn from(pe: ParseError) -> AttributeError {
        AttributeError::Parse(pe)
    }
}

impl<'a> From<BasicParseError<'a>> for AttributeError {
    fn from(e: BasicParseError<'_>) -> AttributeError {
        AttributeError::from(ParseError::from(e))
    }
}

#[derive(Clone)]
pub enum RenderingError {
    Cairo(cairo::Status),
    CircularReference,
    InstancingLimit,
}

impl From<cairo::Status> for RenderingError {
    fn from(e: cairo::Status) -> RenderingError {
        assert!(e != cairo::Status::Success);

        RenderingError::Cairo(e)
    }
}

#[cfg(test)]
pub fn is_parse_error<T>(r: &Result<T, AttributeError>) -> bool {
    match *r {
        Err(AttributeError::Parse(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
pub fn is_value_error<T>(r: &Result<T, AttributeError>) -> bool {
    match *r {
        Err(AttributeError::Value(_)) => true,
        _ => false,
    }
}
