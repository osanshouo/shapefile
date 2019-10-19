use crate::shape::{Shape, BoundingBox};
use crate::error::ShapefileError;

#[derive(Debug, Clone)]
pub struct ShpReader<R> {
    reader: R,
    bb: BoundingBox,
    shape: i32,
    zlim: [f64;2],
    mlim: [f64;2],
    rem_size: i32,
}

use std::io::Read;
impl<R: Read> ShpReader<R> {
    pub fn new(mut reader: R) -> Result<Self, Box<dyn std::error::Error>> {
        let file_code = load_i32_be(&mut reader)?;
        if file_code != 9994 {
            return Err(Box::new(ShapefileError::InvalidFile));
        }

        for _ in 0..5 {
            if load_i32_be(&mut reader)? != 0 {
               return Err(Box::new(ShapefileError::InvalidFile));
            }
        }

        let file_length = load_i32_be(&mut reader)?;

        let version: i32 = load_i32_le(&mut reader)?;
        if version != 1000 {
            return Err(Box::new(ShapefileError::InvalidFile));
        }

        let shape = load_i32_le(&mut reader)?;

        let bb = BoundingBox::load(&mut reader)?;
        let zmin = load_f64(&mut reader)?;
        let zmax = load_f64(&mut reader)?;
        let mmin = load_f64(&mut reader)?;
        let mmax = load_f64(&mut reader)?;

        Ok(ShpReader { 
            reader, bb, shape, 
            zlim: [zmin, zmax], mlim: [mmin, mmax], 
            rem_size: file_length - 50 
        })
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.bb.clone()
    }

    fn get(&mut self) -> Result<Shape, Box<dyn std::error::Error>> {
        //****** record header ******//
        let _record_number = load_i32_be(&mut self.reader)?;
        let content_length = load_i32_be(&mut self.reader)?;

        self.rem_size -= content_length + 4;

        //****** record content ******//
        let shape_type = load_i32_le(&mut self.reader)?;

        let record: Shape = match shape_type {
            0 => Shape::NullShape,
            1 => {
                let x = load_f64(&mut self.reader)?;
                let y = load_f64(&mut self.reader)?;
                Shape::Point{ x, y }
            },
            3 => {
                let bb = BoundingBox::load(&mut self.reader)?;
                let num_parts = load_i32_le(&mut self.reader)? as usize;
                let num_points = load_i32_le(&mut self.reader)? as usize;

                let mut parts = Vec::with_capacity(num_parts);
                for _ in 0..num_parts {
                    parts.push(load_i32_le(&mut self.reader)? as usize);
                }
                assert_eq!( num_parts, parts.len() );

                let mut points = Vec::with_capacity(num_points);
                for _ in 0..num_points {
                    let x = load_f64(&mut self.reader)?;
                    let y = load_f64(&mut self.reader)?;
                    points.push(Shape::Point{ x, y });
                }
                assert_eq!( num_points, points.len() );
                
                assert_eq!( 2*content_length as usize, 44 + 4*num_parts + 16*num_points );
                
                Shape::PolyLine { bb, num_parts, num_points, parts, points }
            },
            5 => {
                let bb = BoundingBox::load(&mut self.reader)?;
                let num_parts = load_i32_le(&mut self.reader)? as usize;
                let num_points = load_i32_le(&mut self.reader)? as usize;

                let mut parts = Vec::with_capacity(num_parts);
                for _ in 0..num_parts {
                    parts.push(load_i32_le(&mut self.reader)? as usize);
                }
                assert_eq!( num_parts, parts.len() );

                let mut points = Vec::with_capacity(num_points);
                for _ in 0..num_points {
                    let x = load_f64(&mut self.reader)?;
                    let y = load_f64(&mut self.reader)?;
                    points.push(Shape::Point{ x, y });
                }
                assert_eq!( num_points, points.len() );
                
                assert_eq!( 2*content_length as usize, 44 + 4*num_parts + 16*num_points );
                
                Shape::Polygon { bb, num_parts, num_points, parts, points }
            },
            8 => {
                let bb = BoundingBox::load(&mut self.reader)?;
                let num_points = load_i32_le(&mut self.reader)? as usize;
                let mut points = Vec::with_capacity(num_points);

                for _ in 0..num_points {
                    let x = load_f64(&mut self.reader)?;
                    let y = load_f64(&mut self.reader)?;
                    points.push( Shape::Point{ x, y } );
                }
                
                Shape::MultiPoint{ bb, num_points, points }
            },
            11 => {
                let x = load_f64(&mut self.reader)?;
                let y = load_f64(&mut self.reader)?;
                let z = load_f64(&mut self.reader)?;
                let m = load_f64(&mut self.reader)?;
                Shape::PointZ { x, y, z, m }
            },
        //     // 13 => Shape::PolyLineZ,
        //     // 15 => Shape::PolygonZ,
        //     // 18 => Shape::MultiPointZ,
            21 => {
                let x = load_f64(&mut self.reader)?;
                let y = load_f64(&mut self.reader)?;
                let m = load_f64(&mut self.reader)?;
                Shape::PointM { x, y, m }
            },
            23 => {
                let bb = BoundingBox::load(&mut self.reader)?;
                let num_parts = load_i32_le(&mut self.reader)? as usize;
                let num_points = load_i32_le(&mut self.reader)? as usize;

                let mut parts = Vec::with_capacity(num_parts);
                for _ in 0..num_parts {
                    parts.push( load_i32_le(&mut self.reader)? as usize );
                }

                let mut points = Vec::with_capacity(num_points);
                for _ in 0..num_points {
                    let x = load_f64(&mut self.reader)?;
                    let y = load_f64(&mut self.reader)?;
                    points.push( Shape::Point{ x, y } );
                }

                if content_length as usize * 2 == 40 + 4*num_parts + 16*(num_points) {
                    Shape::PolyLineM { 
                        bb, num_parts, num_points, parts, points,
                        m_range: [0.0, 0.0], m_array: vec![0.0; num_points] 
                    }
                } else {
                    let m_min = load_f64(&mut self.reader)?;
                    let m_max = load_f64(&mut self.reader)?;

                    let mut m_array = Vec::new();
                    for _ in 0..num_points {
                        m_array.push( load_f64(&mut self.reader)? );
                    }

                    Shape::PolyLineM { 
                        bb, num_parts, num_points, parts, points, m_range: [m_min, m_max], m_array
                    }
                }
            },
            25 => {
                let bb = BoundingBox::load(&mut self.reader)?;
                let num_parts = load_i32_le(&mut self.reader)? as usize;
                let num_points = load_i32_le(&mut self.reader)? as usize;

                let mut parts = Vec::with_capacity(num_parts);
                for _ in 0..num_parts {
                    parts.push( load_i32_le(&mut self.reader)? as usize );
                }

                let mut points = Vec::with_capacity(num_points);
                for _ in 0..num_points {
                    let x = load_f64(&mut self.reader)?;
                    let y = load_f64(&mut self.reader)?;
                    points.push( Shape::Point{ x, y } );
                }

                if content_length as usize * 2 == 40 + 4*num_parts + 16*(num_points) {
                    Shape::PolygonM { 
                        bb, num_parts, num_points, parts, points,
                        m_range: [0.0, 0.0], m_array: vec![0.0; num_points] 
                    }
                } else {
                    let m_min = load_f64(&mut self.reader)?;
                    let m_max = load_f64(&mut self.reader)?;

                    let mut m_array = Vec::new();
                    for _ in 0..num_points {
                        m_array.push( load_f64(&mut self.reader)? );
                    }

                    Shape::PolygonM { 
                        bb, num_parts, num_points, parts, points, m_range: [m_min, m_max], m_array
                    }
                }
            },
            28 => {
                let bb = BoundingBox::load(&mut self.reader)?;
                let num_points = load_i32_le(&mut self.reader)? as usize;

                let mut points = Vec::with_capacity(num_points);
                for _ in 0..num_points {
                    let x = load_f64(&mut self.reader)?;
                    let y = load_f64(&mut self.reader)?;
                    points.push( Shape::Point{ x, y } );
                }

                if content_length as usize*2 == 40 + 16*(num_points) {
                    Shape::MultiPointM{ 
                        bb, num_points, points, 
                        m_range: [0.0, 0.0], m_array: vec![0.0; num_points] 
                    }
                } else {          
                    let m_min = load_f64(&mut self.reader)?;
                    let m_max = load_f64(&mut self.reader)?;

                    let mut m_array = Vec::new();
                    for _ in 0..num_points {
                        m_array.push( load_f64(&mut self.reader)? );
                    }

                    Shape::MultiPointM{ bb, num_points, points, m_range: [m_min, m_max], m_array }
                }
            },
        //     // 31 => Shape::MultiMatch,
            _ => { 
                self.rem_size = 0;
                return Err(Box::new(ShapefileError::InvalidShapeType)); 
            },
        };

        Ok(record)
    }
}

use std::iter::Iterator;

impl<R: Read> Iterator for ShpReader<R> {
    type Item = Result<Shape, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Result<Shape, Box<dyn std::error::Error>>> {
        #[cfg(debug_assertion)]
        { if self.rem_size < 0 { panic!("Parsing the shapefile failed!"); } }

        if self.rem_size == 0 {
            None
        } else {
            Some(self.get())
        }
    }
}


#[inline]
fn load_i32_be<R: Read>(reader: &mut R) -> Result<i32, std::io::Error> {
    let mut buf = [0u8; 4];
    reader.read(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

#[inline]
fn load_i32_le<R: Read>(reader: &mut R) -> Result<i32, std::io::Error> {
    let mut buf = [0u8; 4];
    reader.read(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

#[inline]
fn load_f64<R: Read>(reader: &mut R) -> Result<f64, std::io::Error> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_bits(u64::from_le_bytes(buf)))
}


#[cfg(test)]
mod tests {
    use crate::reader::ShpReader;
    use std::fs::File;
    use std::io::BufReader;
    
    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        // let name = "ne_10m_coastline";
        // let name = "ne_10m_admin_0_countries";
        // let name = "ne_50m_coastline";
        let name = "ne_110m_coastline";

        let file = {
            let file = format!("../data/{0}/{0}.shp", name);
            File::open(&file)?
        };
        let reader = BufReader::new(file);
        let reader = ShpReader::new(reader)?;
    
        // for shp in reader.take(4) {
        //     println!("{:?}", shp);
        // }
        println!("N = {}", reader.count());
        
        Ok(())
    }
}
