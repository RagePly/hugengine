use crate::NumberOfFields;

pub struct Color { 
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl NumberOfFields for Color {
    fn nr_fields(&self) -> usize {
        return 3;
    }
}

impl Color {
    pub fn iter(&self) -> std::vec::IntoIter<f32> {
        vec!(
            self.r,
            self.g,
            self.b,
        ).into_iter()
    }
}

#[macro_export]
macro_rules! col {
    () => {
        col!(black)
    };
    (black) => {
        col!(0.0,0.0,0.0)
    };
    (red) => {
        col!(1.0,0.0,0.0)
    };
    (green) => {
        col!(0.0,1.0,0.0)
    };
    (blue) => {
        col!(0.0,0.0,1.0)
    };
    (white) => {
        col!(1.0,1.0,1.0)
    };
    ($r:expr, $g:expr, $b:expr) => {
        Color {
            r: $r,
            g: $g,
            b: $b,
        }
    };
}
