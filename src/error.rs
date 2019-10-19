#[derive(Debug, Clone, PartialEq)]
pub enum ShapefileError {
    InvalidShapeType,
    InvalidFile,
}

use std::fmt;
impl fmt::Display for ShapefileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShapefileError::InvalidShapeType => write!(f, "Invalid Shape Type"),
            ShapefileError::InvalidFile => write!(f, "Invalid File"),
        }
    }
}

impl std::error::Error for ShapefileError {
}
