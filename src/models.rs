use std::collections::HashMap;
use std::iter::IntoIterator;
use crate::geospace;
use crate::color;
use crate::material;
use crate::NumberOfFields;


pub const SPHERE_ID: i32 = 0;
pub const BOX_ID: i32 = 1;
/// GUID used to reference a registred object
pub type ModelId = u32;

/// Contains data about the object size, variable
pub enum ModelType {
    Sphere, // radii of 1.0
    Box(f32, f32, f32), // dimensions
}

impl ModelType {
    pub fn get_id(&self) -> i32 {
        match self {
            ModelType::Sphere => SPHERE_ID,
            ModelType::Box(_,_,_) => BOX_ID,
        }
    }

    pub fn iter(&self) -> std::vec::IntoIter<f32> {
        match self {
            ModelType::Sphere => vec!(),
            ModelType::Box(w,h,d) => vec!(*w,*h,*d),
        }.into_iter()
    }
}

impl NumberOfFields for ModelType {
    /// Returns number of extra items allocated
    fn nr_fields(&self) -> usize {
        match self {
            ModelType::Sphere => 0,
            ModelType::Box(_,_,_) => 3,
        }
    }
}

/// The properties fully describing each object
pub struct ModelProperty {
    pub t: ModelType,
    pub tf: geospace::Transform,
    pub color: color::Color,
    pub material: material::Material,
}

impl NumberOfFields for ModelProperty {
    fn nr_fields(&self) -> usize {
        self.t.nr_fields() + self.tf.nr_fields() + 
            self.color.nr_fields() + self.material.nr_fields()
    }
}

// TODO: Maintain some upper limit on objects
/// Maintains a registry of each object in the scene.
/// Creates data to fill a shader storage buffer
pub struct ModelManager {
    /// Maintaining data
    registry: HashMap<ModelId, ModelProperty>,
    counter: ModelId,
    modif: bool,
}

impl ModelManager {
    pub fn new() -> Self {
        ModelManager {
            registry: HashMap::new(),
            counter: 0,
            modif: true,
        }
    }

    
    pub fn add_new(&mut self, model: ModelProperty) -> ModelId {
        self.modif = true;
        self.registry.insert(self.counter, model);
        self.counter += 1;
        self.counter
    }
    
    /// Creates shader storage buffers
    pub fn create_ss_buffers(&mut self) -> (Vec<i32>,Vec<f32>) {
        self.modif = false;
        let mut index = 0;
        let mut keys = Vec::new();
        let mut prop = Vec::new();
        for (_, model) in self.registry.iter() {
            keys.push(model.t.get_id());
            keys.push(index);

            model.tf.iter().for_each(|i| prop.push(i));
            model.color.iter().for_each(|i| prop.push(i));
            model.material.iter().for_each(|i| prop.push(i));
            model.t.iter().for_each(|i| prop.push(i));

            index += model.tf.nr_fields() as i32;
            index += model.color.nr_fields() as i32;
            index += model.material.nr_fields() as i32;
            index += model.t.nr_fields() as i32;
        }

        (keys, prop)
    }
}
