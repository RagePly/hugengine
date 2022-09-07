extern crate yaml_rust;
extern crate cgmath;

use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::scanner::ScanError;
use super::{ModelManager, ModelProperty, ModelType, CameraProperty};
use crate::geospace::Transform;
use crate::color::Color;
use crate::material::Material;
use crate::{transform, col, define_material};

use std::fmt;
use std::fmt::{Display, Formatter};
use std::error;
use std::str::FromStr;

use cgmath::Vector3;

#[derive(Debug, Clone, Copy)]
pub enum YamlType {
    Real,
    Integer,
    String,
    Boolean,
    Array,
    Hash,
    Alias,
    Null,
    BadValue,
}

impl From<&Yaml> for YamlType {
    fn from(node: &Yaml) -> Self {
        match node {
            Yaml::Real(_)    => Self::Real,
            Yaml::Integer(_) => Self::Integer,
            Yaml::String(_)  => Self::String,
            Yaml::Boolean(_) => Self::Boolean,
            Yaml::Array(_)   => Self::Array,
            Yaml::Hash(_)    => Self::Hash,
            Yaml::Alias(_)   => Self::Alias,
            Yaml::Null       => Self::Null,
            Yaml::BadValue   => Self::BadValue,
        }
    }
}

impl Display for YamlType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Real => write!(f, "real"),
            Self::Integer => write!(f, "integer"),
            Self::String => write!(f, "string"),
            Self::Boolean => write!(f, "boolean"),
            Self::Array => write!(f, "array"),
            Self::Hash => write!(f, "hash"),
            Self::Alias => write!(f, "alias"),
            Self::Null => write!(f, "null"),
            Self::BadValue => write!(f, "bad-value"),
        }
    }
}

fn display_yaml(value: &Yaml) -> String {
    match value {
        Yaml::Real(s) => s.clone(),
        Yaml::Integer(i) => i.to_string(),
        Yaml::String(s)  => format!("\"{}\"", s),
        Yaml::Boolean(b) => String::from(if *b {"true"} else {"false"}),
        Yaml::Array(_)   => String::from("[...]"),
        Yaml::Hash(_)    => String::from("{...}"),
        Yaml::Alias(_)   => String::from("~Alias~"),
        Yaml::Null       => String::from("null"),
        Yaml::BadValue   => String::from("BAD_VALUE"),
    }
}

#[derive(Debug)]
pub enum ParserError {
    BadKey(String),
    BadType(String, YamlType, YamlType),
    BadValue(String, String),
    MissingKey(String),
    BadDocument(String),
    YamlError(ScanError),
}

impl From<ScanError> for ParserError {
    fn from(se: ScanError) -> Self {
        Self::YamlError(se)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::BadKey(s) => write!(f, "unknown key {}", s),
            Self::BadType(s, t_given, t_expected) => {
                write!(f, "bad type ({}), got {} but expected {}", s, t_given, t_expected)
            }
            Self::BadValue(s, reason) => write!(f, "bad value {}: {}", s, reason),
            Self::MissingKey(s) => write!(f, "necessary key {} not found", s),
            Self::BadDocument(s) => write!(f, "bad document: {}", s),
            Self::YamlError(se) => write!(f, "yaml-rust error: {}", se),
        }
    }
}

impl error::Error for ParserError {}

pub type ParserResult<T> = Result<T, ParserError>;

pub fn parse_scene(source: &str) -> ParserResult<(ModelManager, CameraProperty)> {
    // load using yaml
    let docs = YamlLoader::load_from_str(source)?;

    if docs.len() != 1 {
        return Err(ParserError::BadDocument("only a single yaml document should be used".to_string()));
    }

    let doc = &docs[0];
    let source_hash = doc
                      .as_hash()
                      .ok_or(ParserError::BadType(String::from("entire document"), doc.into(), YamlType::Hash))?;
    
    if source_hash.len() != 1 {
        return Err(ParserError::BadDocument("only one key-value pair at top scope".to_string()));
    }

    // get scene
    let scene = source_hash
                 .get(&Yaml::String("scene".to_owned()))
                 .ok_or_else(|| 
                        if let Some((key, _)) = source_hash.front() {
                            match key {
                                Yaml::String(s) => ParserError::BadKey(s.clone()),
                                other => ParserError::BadType(display_yaml(other), other.into(), YamlType::String),
                            }
                        } else {
                            unreachable!() // i guarantee that at least one element exists
                        }
                     )
                 ?;

    let scene_hash = scene.as_hash().ok_or(ParserError::BadType(display_yaml(scene), scene.into(), YamlType::Hash))?;
    let mut mm = ModelManager::new();
    let mut cp = CameraProperty::new();
    for (key, val) in scene_hash.iter() {
        let key_str = key.as_str().ok_or(ParserError::BadType(display_yaml(key), key.into(), YamlType::String))?;

        match key_str {
            "models" => {
                let models = parse_models(val)?;
                models.into_iter().for_each(|m| { mm.add_new(m); });
            }
            "camera" => {
                let (cpos, cpitch, cyaw) = parse_camera(val)?;    
                cp.tf.x = cpos.x;
                cp.tf.y = cpos.y;
                cp.tf.z = cpos.z;
                cp.tf.pitch = cpitch;
                cp.tf.head = cyaw;
            },
            s => { return Err(ParserError::BadKey(String::from(s))); },
        }
    }
    Ok((mm, cp))
}

fn parse_models(node: &Yaml) -> ParserResult<Vec<ModelProperty>> {
    let mut modelprops = Vec::new();
    let models = node.as_vec().ok_or(ParserError::BadType(display_yaml(node), node.into(), YamlType::Array))?;

    for model in models.iter() {
        let mut model_type: Option<ModelType> = None;
        let mut model_tf = transform!();
        let mut model_color = col!();
        let mut model_mat = define_material!(1.0);

        // verify that model is a hash
        let model_hash = model.as_hash().ok_or(ParserError::BadType(display_yaml(model), model.into(), YamlType::Hash))?;
        for (property, value) in model_hash.iter() {
            // verify that property is a string type
            let property_str = property
                               .as_str()
                               .ok_or(ParserError::BadType(display_yaml(property), property.into(), YamlType::String))?;

            match property_str {
                "type" => { model_type = Some(parse_type(value)?); }
                "transform" => { model_tf = parse_transform(value)?; }
                "color" => { model_color = parse_color(value)?; }
                "material" => { model_mat = parse_material(value)?; }
                badkey => { return Err(ParserError::BadKey(badkey.to_owned())); },
            }
        }

        modelprops.push(ModelProperty {
            t: model_type.ok_or(ParserError::MissingKey(String::from("type")))?,
            tf: model_tf,
            color: model_color,
            material: model_mat,
        });
    }
    Ok(modelprops)
}

fn parse_type(value: &Yaml) -> ParserResult<ModelType> {
    let s = value.as_str().ok_or(ParserError::BadType(display_yaml(value), value.into(), YamlType::String))?;
    s.try_into().or(Err(ParserError::BadValue(s.to_owned(), String::from("type does not exist"))))
}

fn parse_transform(value: &Yaml) -> ParserResult<Transform> {
    let transform_hash = value.as_hash().ok_or(ParserError::BadType(display_yaml(value), value.into(), YamlType::Hash))?;    
    let mut tf = transform!();

    for (property, value) in transform_hash.iter() {
        let property_str = property
                           .as_str()
                           .ok_or(ParserError::BadType(display_yaml(property), property.into(), YamlType::String))?;
        match property_str {
            "pos" => {
                let v = parse_vector(value)?;
                if v.len() != 3 {
                    return Err(ParserError::BadValue(format!("{:?}", v), String::from("position must be a 3-component vector")));
                }
                tf.x = v[0];
                tf.y = v[1];
                tf.z = v[2];
            }
            "scale" => { tf.scale = parse_real(value)?; }
            "orientation" => {
                let (head, pitch, roll) = parse_orientation(value)?;
                tf.head = head;
                tf.pitch = pitch;
                tf.rotate = roll;
            }
            _ => { return Err(ParserError::BadKey(property_str.to_owned())); },
        }
    }
    Ok(tf)
}

fn parse_real(value: &Yaml) -> ParserResult<f32> {
    if let Yaml::Real(s) = value {
        f32::from_str(s.as_str())
        .or(Err(ParserError::BadValue(
                    s.clone(),
                    String::from("value can't fit into mediump float"))))
    } else {
        Err(ParserError::BadType(display_yaml(value), value.into(), YamlType::Real))
    }
}

fn parse_vector(value: &Yaml) -> ParserResult<Vec<f32>> {
    let arr = value.as_vec().ok_or(ParserError::BadType(display_yaml(value), value.into(), YamlType::Array))?;
    let mut v_out = Vec::new();
    for v in arr {
        v_out.push(parse_real(v)?);
    }
    Ok(v_out)
}

fn parse_orientation(value: &Yaml) -> ParserResult<(f32, f32, f32)> {
    let mut head: f32 = 0.0; 
    let mut pitch: f32 = 0.0; 
    let mut roll: f32 = 0.0; 

    let orient_hash = value.as_hash().ok_or(ParserError::BadType(display_yaml(value), value.into(), YamlType::Hash))?;    
    for (property, value) in orient_hash.iter() {
        let property_str = property
                           .as_str()
                           .ok_or(ParserError::BadType(display_yaml(property), property.into(), YamlType::String))?;
        match property_str {
            "head" => { head = parse_real(value)?; }
            "pitch" => { pitch = parse_real(value)?; }
            "roll" => { roll = parse_real(value)?; }
            _ => { return Err(ParserError::BadKey(property_str.to_owned())); },
        }
    }
    Ok((head, pitch, roll))
}

fn parse_material(value: &Yaml) -> ParserResult<Material> {
    let material_hash = value.as_hash().ok_or(ParserError::BadType(display_yaml(value), value.into(), YamlType::Hash))?;    
    let mut m = define_material!(1.0);

    for (property, value) in material_hash.iter() {
        let property_str = property
                           .as_str()
                           .ok_or(ParserError::BadType(display_yaml(property), property.into(), YamlType::String))?;
        match property_str {
            "reflectance" => { m.reflectance = parse_real(value)?; }
            _ => { return Err(ParserError::BadKey(property_str.to_owned())); },
        }
    }
    Ok(m)

}

fn parse_color(value: &Yaml) -> ParserResult<Color> {
    match value {
        Yaml::String(col) => Color::from_str(col.as_str()).or(Err(ParserError::BadValue(display_yaml(value), String::from("color is invalid")))),
        other => {
            let v = parse_vector(other)?;
            if v.len() != 3 {
                Err(ParserError::BadValue(display_yaml(value), String::from("color must be a 3-component vector")))
            } else {
                Ok(Color {
                    r: v[0],
                    g: v[1],
                    b: v[2],
                })
            }
        }
    }
}

fn parse_camera(node: &Yaml) -> ParserResult<(Vector3<f32>, f32, f32)> {
    let mut position = Vector3::<f32>::new(0.0, 0.0, 0.0);
    let mut pitch = 0.0;
    let mut yaw = 0.0;

    let camera_hash = node.as_hash().ok_or(ParserError::BadType(display_yaml(node), node.into(), YamlType::Hash))?;

    for (property, value) in camera_hash.iter() {
        let property_str = property
                           .as_str()
                           .ok_or(ParserError::BadType(display_yaml(property), property.into(), YamlType::String))?;
        match property_str {
            "position" => {
                let v = parse_vector(value)?;
                if v.len() != 3 {
                    return Err(ParserError::BadValue(display_yaml(value), String::from("camera position must be a 3-component vector")));
                } else {
                    position.x = v[0];
                    position.y = v[1];
                    position.z = v[2];
                }
            }
            "pitch" => { pitch = parse_real(value)?; }
            "yaw" => { yaw = parse_real(value)?; }
            _ => { return Err(ParserError::BadKey(property_str.to_owned())); },
        }
    }

    Ok((position, pitch, yaw))
}

#[cfg(test)]
mod tests {
    const TESTFILE: &'static str = "scenes/test.yaml";
    #[test]
    fn test_file_exists() {
        use std::fs::File;
        let file = File::open(TESTFILE);
        assert!(file.is_ok());
    }

    #[test]
    fn light_check_on_format() {
        use std::fs::read_to_string;
        use yaml_rust::{Yaml, YamlLoader};
        use std::str::FromStr;

        let source = read_to_string(TESTFILE).expect("file should exists");
        let docs = YamlLoader::load_from_str(source.as_str())
            .expect("yaml document correctly formatted");
        assert_eq!(docs.len(), 1, "test-file should only be one document");
        let scene = &docs[0];

        // Test field access
        assert_eq!(scene["scene"]["models"][0]["type"].as_str().unwrap(), "Sphere");
        let pitch = &scene["scene"]["models"][0]["transform"]["orientation"]["pitch"];
        if let Yaml::Real(ref s) = *pitch {
            assert_eq!(s, "3.14");
            let _pitch_num = f32::from_str(s).expect("parsing from float works");
        }

        let sphere = scene["scene"]["models"][0]["transform"]["pos"].as_vec().unwrap();
        assert_eq!(sphere.len(), 3);
    }

    #[test]
    fn verify_parse() {
        use super::parse_scene;
        use crate::{col, transform, define_material};
        use crate::material::Material;
        use crate::geospace::Transform;
        use crate::color::Color;
        use crate::models::{ModelManager, ModelProperty, ModelType};
        use std::fs::read_to_string;
        use std::str::FromStr;
        
        let source = read_to_string(TESTFILE).expect("file should exists");
        let (models, _)  = parse_scene(source.as_str()).expect("parse is successfull");
        assert_eq!(models.len(), 2);

        let sphere = ModelProperty {
            t: ModelType::Sphere,
            tf: transform!(0.0, 0.0, -10.0, 1.0, 0.0, 3.14, 0.0),
            color: col!(1.0, 0.0, 0.0),
            material: define_material!(1.0),
        };
        let plane = ModelProperty {
            t: ModelType::Plane,
            tf: transform!(0.0, -10.0, 0.0, 1.0, 0.0, 0.0, 0.0),
            color: Color::from_str("white").expect("white should exists"),
            material: define_material!(1.0),
        };
        

        assert_eq!(models[&0], sphere);
        assert_eq!(models[&1], plane);
    }
}


