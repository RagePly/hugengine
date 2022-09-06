extern crate yaml_rust;

use yaml_rust::{Yaml, YamlLoader};

// TODO: implement parser haha

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

}


