use crate::NumberOfFields;

pub struct Transform {
    // Position
    pub x: f32,
    pub y: f32,
    pub z: f32,

    // Scaling
    pub scale: f32,

    // Rotation
    pub roti: f32,
    pub rotj: f32,
    pub rotk: f32
}

impl NumberOfFields for Transform {
    fn nr_fields(&self) -> usize {
        7
    }
}

impl Transform {
    pub fn iter(&self) -> std::vec::IntoIter<f32> {
        vec!(
            self.x,
            self.y,
            self.z,
            self.scale,
            self.roti,
            self.rotj,
            self.rotk,
        ).into_iter()
    }
}

#[macro_export]
macro_rules! transform {
    () => {
        transform!(0.0,0.0,0.0);
    };
    ($x:expr, $y:expr, $z:expr) => {
        transform!($x, $y, $z, 1.0, 0.0, 0.0, 0.0);
    };
    ($x:expr, $y:expr, $z:expr, $s:expr) => {
        transform!($x, $y, $z, $s, 0.0, 0.0, 0.0);
    };
    ($x:expr, $y:expr, $z:expr, $s:expr, $i:expr, $j:expr, $k:expr) => {
        Transform {
            x: $x,
            y: $y,
            z: $z,
            scale: $s,
            roti: $i,
            rotj: $j,
            rotk: $k,
        }
    };
}
