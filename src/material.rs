use crate::NumberOfFields;

pub struct Material {
    pub reflectance: f32
}

impl NumberOfFields for Material {
    fn nr_fields(&self) -> usize {
        return 1;
    }
}

impl Material {
    pub fn iter(&self) -> std::vec::IntoIter<f32> {
        vec!(
            self.reflectance,
        ).into_iter()
    }
}

#[macro_export]
macro_rules! define_material {
    ($ref:expr) => {
        Material {
            reflectance: $ref
        }
    }
}
