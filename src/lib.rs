pub fn foo() -> String {
    String::from("hello")
}

trait NumberOfFields {
    fn nr_fields(&self) -> usize;
}

pub mod color;
pub mod material;
pub mod geospace;
pub mod models;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    // this test should be ran a few times, there is a hash algorithm in there
    // that randomizes the results.
    // #[test]
    fn creating_ss_buffers() {
        use crate::models;
        use crate::models::{ModelManager, ModelType, ModelProperty};
        use crate::color::Color;
        use crate::material::Material;
        use crate::geospace::Transform;
        use crate::{col, transform, define_material};

        let mut modelmanager = ModelManager::new();
        // a white, mirror ball located at the origin
        let white_ball = modelmanager.add_new(ModelProperty {
            t: ModelType::Sphere,
            tf: transform!(),
            color: col!(white),
            material: define_material!(1.0),
        });

        assert_eq!(white_ball, 1);

        // a red, opaque ball above the sphere
        let red_box = modelmanager.add_new(ModelProperty {
            t: ModelType::Box(1.0,1.0,1.0),
            tf: transform!(0.0,1.0,0.0),
            color: col!(red),
            material: define_material!(0.0),
        });

        assert_eq!(red_box, 2);

        let (ids, props) = modelmanager.create_ss_buffers();

        // this is just the order that the iterator traverses the hashmap and should
        // not be seen as "expected behaviour". What is important is that the correct
        // info ends up where it is expected "some of the time"
        assert_eq!(ids, vec![
            models::BOX_ID,
            0,
            models::SPHERE_ID, 
            14,
        ]);

        assert_eq!(props, vec![
            // red box
            0.0,1.0,0.0,    // position (positive y)
            1.0,            // scale (1x)
            0.0,0.0,0.0,    // rotation (none)
            1.0,0.0,0.0,    // color (red)
            0.0,            // no reflectance
            1.0,1.0,1.0,    // sides 1x1x1
            // sphere
            0.0,0.0,0.0,    // position (origin)
            1.0,            // scale (1x)
            0.0,0.0,0.0,    // rotation (none)
            1.0,1.0,1.0,    // color (white)
            1.0,            // total reflectance
                            // no extra fields
        ]);
    }
}
