#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    NullShape,
    Point {
        x: f64, y: f64,
    },
    PolyLine {
        bb: BoundingBox,
        num_parts: usize,
        num_points: usize,
        parts: Vec<usize>,
        points: Vec<Shape>,
    },
    Polygon {
        bb: BoundingBox,
        num_parts: usize,
        num_points: usize,
        parts: Vec<usize>,
        points: Vec<Shape>,
    },
    MultiPoint {
        bb: BoundingBox,
        num_points: usize,
        points: Vec<Shape>,
    },
    PointZ {
        x: f64, y: f64, z: f64, m: f64,
    },
    PolyLineZ,
    PolygonZ,
    MultiPointZ,
    PointM {
        x: f64, y: f64, m: f64,
    },
    PolyLineM {
        bb: BoundingBox,
        num_parts: usize,
        num_points: usize,
        parts: Vec<usize>,
        points: Vec<Shape>,
        m_range: [f64; 2],
        m_array: Vec<f64>,
    },
    PolygonM {
        bb: BoundingBox,
        num_parts: usize,
        num_points: usize,
        parts: Vec<usize>,
        points: Vec<Shape>,
        m_range: [f64; 2],
        m_array: Vec<f64>,
    },
    MultiPointM {
        bb: BoundingBox,
        num_points: usize,
        points: Vec<Shape>,
        m_range: [f64;2],
        m_array: Vec<f64>,
    },
    MultiMatch,
}

// use crate::error::ShapefileError;
// use std::convert::{TryFrom, Into};

// impl TryFrom<i32> for Shape {
//     type Error = ShapefileError;

//     fn try_from(value: i32) -> Result<Self, Self::Error> {
//         match value {
//             0 => Ok(Shape::NullShape),
//             1 => Ok(Shape::Point),
//             3 => Ok(Shape::PolyLine),
//             5 => Ok(Shape::Polygon),
//             8 => Ok(Shape::MultiPoint),
//             11 => Ok(Shape::PointZ),
//             13 => Ok(Shape::PolyLineZ),
//             15 => Ok(Shape::PolygonZ),
//             18 => Ok(Shape::MultiPointZ),
//             21 => Ok(Shape::PointM),
//             23 => Ok(Shape::PolyLineM),
//             25 => Ok(Shape::PolygonM),
//             28 => Ok(Shape::MultiPointM),
//             31 => Ok(Shape::MultiMatch),
//             _ => Err(ShapefileError::InvalidShapeType),
//         }
//     }
// }

use std::convert::Into;

impl Into<i32> for Shape {
    fn into(self) -> i32 {
        match self {
            Shape::NullShape => 0,
            Shape::Point { .. } => 1,
            Shape::PolyLine { .. } => 3,
            Shape::Polygon { .. } => 5,
            Shape::MultiPoint { .. } => 8,
            Shape::PointZ { .. } => 11,
            Shape::PolyLineZ => 13,
            Shape::PolygonZ => 15,
            Shape::MultiPointZ => 18,
            Shape::PointM { .. } => 21,
            Shape::PolyLineM { .. } => 23,
            Shape::PolygonM { .. } => 25,
            Shape::MultiPointM { .. } => 28,
            Shape::MultiMatch => 31,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}


use std::io::Read;

impl BoundingBox {
    pub fn load<R: Read>(mut reader: &mut R) -> Result<Self, std::io::Error> {
        let xmin = load_f64(&mut reader)?;
        let ymin = load_f64(&mut reader)?;
        let xmax = load_f64(&mut reader)?;
        let ymax = load_f64(&mut reader)?;

        Ok(BoundingBox { xmin, ymin, xmax, ymax })
    }
}

#[inline]
fn load_f64<R: Read>(reader: &mut R) -> Result<f64, std::io::Error> {
    let mut buf = [0u8; 8];
    reader.read(&mut buf)?;
    Ok(f64::from_bits(u64::from_le_bytes(buf)))
}
