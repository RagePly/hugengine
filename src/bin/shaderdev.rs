extern crate glfw;
extern crate glad_gl;
extern crate cgmath;

use glfw::{Action, Context, Key};
use glad_gl::gl;
use std::ptr;
use std::ffi::CString;
use std::mem;
use std::path::Path;
use std::os::raw::c_void;
use std::io;
use std::fs;
use std::time::{SystemTime, Instant, Duration};

use hugengine::models::parser::parse_scene;

use cgmath::{Vector3, Basis3, Rotation, Rotation3, Rad, Zero, InnerSpace};


const UNIFORM_TIME:          &'static str = "uTime";
const UNIFORM_SCREEN_RES:    &'static str = "uScreenResolution";
const UNIFORM_RATIO:         &'static str = "uRatio";
const UNIFORM_CAMPITCH:      &'static str = "uCamPitch";
const UNIFORM_CAMYAW:        &'static str = "uCamYaw";
const UNIFORM_CAMPOS:        &'static str = "uCamPos";
const UNIFORM_SKYBOXSAMPLER: &'static str = "uSkyBoxSampler";

const CAMERA_MOVEMENTSPEED: f32 = 1.0;  // Units per second
const CAMERA_ROTATIONSPEED: f32 = 1.0;  // Rad per second

const WINDOW_HEIGHT: i32 = 720;
const WINDOW_WIDTH: i32 = 1280;

const PATH_SCENE_TEMPLATE: &'static str = "scenes/template.yaml";

// const UNIFORM_MODEL_INDEX: &'static str = "ModelIndex";
// const UNIFORM_MODEL_PROPS: &'static str = "ModelProperties";

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Fragment,
    Vertex,
}

impl ShaderType {
    pub fn to_str(&self) -> String {
        String::from(
            match self {
                ShaderType::Fragment => "fragment",
                ShaderType::Vertex => "vertex",
            }
        )
    }
}

impl From<ShaderType> for gl::GLuint {
    fn from(st: ShaderType) -> gl::GLuint {
        match st {
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::Vertex => gl::VERTEX_SHADER,
        }
    }
}

#[derive(Debug)]
pub enum CompileProgramError {
    LoadSourceError(io::Error),
    CStringInitilizationError,
    CompileShaderError(ShaderType, String),
    LinkProgramError(String),
}

fn set_uniform1f(id: gl::GLuint, varname: &str, val: f32) -> bool {
    let cstring = match CString::new(varname) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };

    unsafe {
        let location = gl::GetUniformLocation(id, cstring.as_c_str().as_ptr());
        if location != -1 {
            gl::UseProgram(id);
            gl::Uniform1f(location, val);
            return true;
        } else {
            return false;
        }
    }
}

fn set_uniform1i(id: gl::GLuint, varname: &str, v0: gl::GLint) -> bool {
    let cstring = match CString::new(varname) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };

    unsafe {
        let location = gl::GetUniformLocation(id, cstring.as_c_str().as_ptr());
        if location != -1 {
            gl::UseProgram(id);
            gl::Uniform1i(location, v0);
            return true;
        } else {
            return false;
        }
    }
}

fn set_uniform2i(id: gl::GLuint, varname: &str, v0: gl::GLint, v1: gl::GLint) -> bool {
    let cstring = match CString::new(varname) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };

    unsafe {
        let location = gl::GetUniformLocation(id, cstring.as_c_str().as_ptr());
        if location != -1 {
            gl::UseProgram(id);
            gl::Uniform2i(location, v0, v1);
            return true;
        } else {
            return false;
        }
    }
}

fn set_uniform3f(id: gl::GLuint, varname: &str, v0: gl::GLfloat, v1: gl::GLfloat, v2: gl::GLfloat) -> bool {
    let cstring = match CString::new(varname) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };

    unsafe {
        let location = gl::GetUniformLocation(id, cstring.as_c_str().as_ptr());
        if location != -1 {
            gl::UseProgram(id);
            gl::Uniform3f(location, v0, v1, v2);
            return true;
        } else {
            return false;
        }
    }

}

fn compile_shader(shader_type: ShaderType, source: CString) -> Result<gl::GLuint, CompileProgramError> {
    let shader_id: gl::GLuint;
    let mut status: gl::GLint = 0;
    
    // only if the wrong function-pointers have been loades should this fail, as 
    unsafe {
        shader_id = gl::CreateShader(shader_type.into());
        gl::ShaderSource(shader_id, 1, &source.as_ptr(), ptr::null());
        gl::CompileShader(shader_id);
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut status);
    }

    if status == gl::FALSE.into() {
        let log = CString::new([0x61u8; 1024])
            .expect("CString: init from \'a\' ok")
            .into_raw();

        unsafe {
            gl::GetShaderInfoLog(shader_id, 1024, ptr::null_mut(), log);
            gl::DeleteShader(shader_id);

            Err(
                CompileProgramError::CompileShaderError(
                    shader_type,
                    CString::from_raw(log)
                    .into_string()
                    .unwrap_or(
                        String::from(
                            "CString: internal null byte detected"
                        )
                    )
                )
            )
        }
    } else {
        Ok(shader_id)
    }
}

fn compile_program<P: AsRef<Path>>(vert_p: P, frag_p: P) -> Result<gl::GLuint, CompileProgramError> {
    use CompileProgramError::*;

    let vert_source = match fs::read_to_string(vert_p) {
        Ok(s) => match CString::new(s) {
            Ok(cstr) => cstr,
            _ => {
                return Err(CStringInitilizationError)
            }
        }
        Err(e) => {
            return Err(LoadSourceError(e));
        }
    };

    let frag_source = match fs::read_to_string(frag_p) {
        Ok(s) => match CString::new(s) {
            Ok(cstr) => cstr,
            _ => {
                return Err(CStringInitilizationError)
            }
        }
        Err(e) => {
            return Err(LoadSourceError(e));
        }
    };

    let frag_id = match compile_shader(ShaderType::Fragment, frag_source) {
        Ok(i) => i,
        Err(e) => { return Err(e); },
    };

    let vert_id = match compile_shader(ShaderType::Vertex, vert_source) {
        Ok(i) => i,
        Err(e) => { return Err(e); },
    };

    // Link program
    let program_id: gl::GLuint;
    let mut status: gl::GLint = 0;
    unsafe {
        program_id = gl::CreateProgram();
        gl::AttachShader(program_id, frag_id);
        gl::AttachShader(program_id, vert_id);
        gl::LinkProgram(program_id);

        gl::DeleteShader(frag_id);
        gl::DeleteShader(vert_id);
        
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut status);
    }

    if status == gl::FALSE.into() {
        let log = CString::new([0x61u8; 1024])
            .expect("CString: init from \'a\' ok")
            .into_raw();

        unsafe {
            gl::GetProgramInfoLog(program_id, 1024, ptr::null_mut(), log);
            gl::DeleteProgram(program_id);

            Err(
                CompileProgramError::LinkProgramError(
                    CString::from_raw(log)
                    .into_string()
                    .unwrap_or(
                        String::from(
                            "CString: internal null byte detected"
                        )
                    )
                )
            )
        }
    } else {
        Ok(program_id)
    }
}

fn last_modified<P: AsRef<Path>>(path: P) -> io::Result<SystemTime> {
    let metadata = fs::metadata(path)?;
    metadata.modified()
}

// TODO: make these a fallback if not specified
static VERT_SHADER_PATH: &'static str = r#".\shaders\shader.vert"#;
static FRAG_SHADER_PATH: &'static str = r#".\shaders\shader.frag"#;

#[allow(unused_macros)]
macro_rules! callgl {
    ($e:expr) => {
        while gl::GetError() != gl::NO_ERROR {}
        $e;
        {
            let callgl_error = gl::GetError();
            if callgl_error != gl::NO_ERROR {
                println!("opengl error: {},{}: {:?}", file!(), line!(), callgl_error);
            }
        }
    }
}

fn main() {
    use CompileProgramError::*;
    // load GLFW lib and create a window as well as a opengl-target
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let mut window_height = WINDOW_HEIGHT;
    let mut window_width = WINDOW_WIDTH;

    let (mut window, events) = glfw.create_window(
        window_width as u32, 
        window_height as u32, 
        "Shader-dev 0.1", 
        glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_size_polling(true);

    window.make_current();

    // load opengl function-pointers and fetch current version
    gl::load(|e| glfw.get_proc_address_raw(e) as *const std::os::raw::c_void);

    let (mut majorv, mut minorv) = (0,0);
   
    unsafe {
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut majorv);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut minorv);
        gl::Viewport(0,0,window_width,window_height);
    }
    println!("using OpenGL version {}.{}", majorv, minorv);

    // vertex-buffer-object, vertex-array-object and ellement-array-buffer-object
    let mut vbo: gl::GLuint = 0;
    let mut vao: gl::GLuint = 0;
    let mut ebo: gl::GLuint = 0;
    let mut index_ssbo: gl::GLuint = 0;
    let mut props_ssbo: gl::GLuint = 0;
    let mut skybox_texobj: gl::GLuint = 0;
    
    // A square that fills the screen
    let vertices: [gl::GLfloat; 12] = [
         1.0,  1.0, 0.0,  // top right
         1.0, -1.0, 0.0,  // bottom right
        -1.0, -1.0, 0.0,  // bottom left
        -1.0,  1.0, 0.0,  // top left
    ];  

    let indices: [gl::GLuint; 6] = [
        0, 1, 3,
        1, 2, 3,
    ];

    let mut sides: Vec<Vec<f32>> = (0..6).into_iter().map(|_| (0..256*256*3).into_iter().map(|_| 0.0).collect()).collect();
    sides[0].as_mut_slice().chunks_mut(3).for_each(|v| { v[0] = 1.0; });
    sides[1].as_mut_slice().chunks_mut(3).for_each(|v| { v[0] = 0.5; });
    sides[2].as_mut_slice().chunks_mut(3).for_each(|v| { v[1] = 1.0; });
    sides[3].as_mut_slice().chunks_mut(3).for_each(|v| { v[1] = 0.5; });
    sides[4].as_mut_slice().chunks_mut(3).for_each(|v| { v[2] = 1.0; });
    sides[5].as_mut_slice().chunks_mut(3).for_each(|v| { v[2] = 0.5; });

    // Objects
    let scene_source = fs::read_to_string(PATH_SCENE_TEMPLATE).expect("file exists");
    let (mut model_manager, camera_prop) = parse_scene(scene_source.as_str()).expect("scene is correctly formatted");
    let (model_indices, model_properties) = model_manager.create_ss_buffers();

    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut ebo);
        gl::GenBuffers(1, &mut index_ssbo);
        gl::GenBuffers(1, &mut props_ssbo);
        gl::GenTextures(1, &mut skybox_texobj);

        // create skybox
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox_texobj);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        
        // allocate storage
        gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl::RGBA as i32, 256, 256, 0, gl::RGB, gl::FLOAT, 0 as *const c_void);
        gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl::RGBA as i32, 256, 256, 0, gl::RGB, gl::FLOAT, 0 as *const c_void);
        gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl::RGBA as i32, 256, 256, 0, gl::RGB, gl::FLOAT, 0 as *const c_void);
        gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl::RGBA as i32, 256, 256, 0, gl::RGB, gl::FLOAT, 0 as *const c_void);
        gl::TexImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl::RGBA as i32, 256, 256, 0, gl::RGB, gl::FLOAT, 0 as *const c_void);
        gl::TexImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl::RGBA as i32, 256, 256, 0, gl::RGB, gl::FLOAT, 0 as *const c_void);

        // write texture
        gl::TexSubImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X, 0, 0, 0, 256, 256, gl::RGB, gl::FLOAT, sides[0].as_ptr() as *const c_void);
        gl::TexSubImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_X, 0, 0, 0, 256, 256, gl::RGB, gl::FLOAT, sides[1].as_ptr() as *const c_void);
        gl::TexSubImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Y, 0, 0, 0, 256, 256, gl::RGB, gl::FLOAT, sides[2].as_ptr() as *const c_void);
        gl::TexSubImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, 0, 0, 256, 256, gl::RGB, gl::FLOAT, sides[3].as_ptr() as *const c_void);
        gl::TexSubImage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_Z, 0, 0, 0, 256, 256, gl::RGB, gl::FLOAT, sides[4].as_ptr() as *const c_void);
        gl::TexSubImage2D(gl::TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, 0, 0, 256, 256, gl::RGB, gl::FLOAT, sides[5].as_ptr() as *const c_void);

        gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);

        gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);



        // store models in buffer
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, index_ssbo);
        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER, 
            (model_indices.len() * mem::size_of::<i32>()).try_into().unwrap(),
            model_indices.as_ptr() as *const c_void, 
            gl::STATIC_DRAW
        );

        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, props_ssbo);
        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER, 
            (model_properties.len() * mem::size_of::<f32>()).try_into().unwrap(),
            model_properties.as_ptr() as *const c_void, 
            gl::STATIC_DRAW
        );

        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);


        // store options in va
        gl::BindVertexArray(vao);

        // store vertices in vbo
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, 
            mem::size_of::<[gl::GLfloat; 12]>() as isize, 
            vertices.as_slice().as_ptr() as *const c_void, 
            gl::STATIC_DRAW,
        );

        // store ordering in ebo
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of::<[gl::GLuint; 6]>() as isize,
            indices.as_slice().as_ptr() as *const c_void,
            gl::STATIC_DRAW,
        );
        
        // add attributes to verticies (no material-constants, just positions)
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<[gl::GLfloat; 3]>() as i32,
            0 as *const c_void,
        );

        gl::EnableVertexAttribArray(0);
    }
    
    // timepoints for use to throttle gpu-time
    let mut vert_last_modified: SystemTime = SystemTime::UNIX_EPOCH;
    let mut frag_last_modified: SystemTime = SystemTime::UNIX_EPOCH;
    let mut last_check: SystemTime = SystemTime::UNIX_EPOCH;
    let mut last_render: Instant = Instant::now();

    let mut program_birth: Instant = Instant::now();
    let mut program_id: Option<gl::GLuint> = None;

    let mut camera_position = Vector3::new(camera_prop.tf.x, camera_prop.tf.y, camera_prop.tf.z);
    let mut camera_pitch: f32 = camera_prop.tf.pitch;
    let mut camera_yaw: f32 = camera_prop.tf.head;

    while !window.should_close() {
        // check every second if program has been updated
        if let Ok(d) = SystemTime::now().duration_since(last_check) {
            if d > Duration::from_secs(1) {
                last_check = SystemTime::now();

                // check if files have been updated
                match (last_modified(VERT_SHADER_PATH), last_modified(FRAG_SHADER_PATH)) {
                    (Ok(vert_time), Ok(frag_time)) => {
                        // recompile program if source has changed since last check
                        if (vert_time != vert_last_modified) | (frag_time != frag_last_modified) {
                            print!("compiling shaders...");
                            match compile_program(VERT_SHADER_PATH, FRAG_SHADER_PATH) {
                                // delete old program if compilation was successfull
                                Ok(new_program_id) => {
                                    if let Some(id) = program_id {
                                        unsafe {gl::DeleteProgram(id);} 
                                    }
                                    program_id = Some(new_program_id);
                                    println!("ok!");
                                    // supply aspect ratio
                                    set_uniform1f(new_program_id, 
                                        UNIFORM_RATIO, 
                                        window_width as f32 / window_height as f32);
                                    // supply screenwidth and -height
                                    set_uniform2i(new_program_id, 
                                        UNIFORM_SCREEN_RES,
                                        window_width, 
                                        window_height);

                                    // supply object data
    // Getting the position GLuint glGetProgramResourceIndex( GLuint program, GL_SHADER_STORAGE_BLOCK, const char *name );
                                    unsafe {
                                        gl::UseProgram(new_program_id);

                                        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, index_ssbo);
                                        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 4, index_ssbo);

                                        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, props_ssbo);
                                        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 5, props_ssbo);

                                        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
                                    }

                                    program_birth = Instant::now();
                                }
                                Err(CompileShaderError(_, error_message)) => {
                                    println!("error!\n{}", error_message);
                                }
                                Err(LinkProgramError(error_message)) => {
                                    println!("error!\n{error_message}");
                                }
                                Err(LoadSourceError(e)) => {
                                    println!("error!\nLoading source files failed due to the following os-error:\ncode: {}\nkind: {:?}",
                                        e.raw_os_error().unwrap_or(0),
                                        e.kind(),
                                    );
                                }
                                Err(CStringInitilizationError) => {
                                    println!("\nfailed to initialize c-string");
                                }
                            }

                            // update timestamp
                            vert_last_modified = vert_time;
                            frag_last_modified = frag_time;
                        }
                    }
                    // couldn't load timestamps from sources, list errors
                    (vert, frag) => {
                        if let Err(vert_error) = vert {
                            println!("failed to load metadata from {VERT_SHADER_PATH}, due to:\n{vert_error:?}");
                        }
                        if let Err(frag_error) = frag {
                            println!("failed to load metadata from {FRAG_SHADER_PATH}, due to:\n{frag_error:?}");
                        }
                    }
                }
            }
        } else {
            println!("A file was last modified in the future...");
        }

        // cap fps at 30
        let delta_time = Instant::now().duration_since(last_render);
        if delta_time > Duration::from_millis(33) {
            let mut camera_posv: Vector3<f32> = Vector3::zero();

            // update camera position
            if window.get_key(Key::W) == Action::Press {
                camera_posv.z = -1.0;
            }

            if window.get_key(Key::A) == Action::Press {
                camera_posv.x = -1.0;
            }

            if window.get_key(Key::S) == Action::Press {
                camera_posv.z = 1.0;
            }

            if window.get_key(Key::D) == Action::Press {
                camera_posv.x = 1.0;
            }

            if window.get_key(Key::Z) == Action::Press {
                camera_posv.y = 1.0;
            }

            if window.get_key(Key::X) == Action::Press {
                camera_posv.y = -1.0;
            }
            

            let dt = delta_time.as_secs_f32();
            let mag = camera_posv.magnitude2();

            if mag > 0.0 {
                camera_posv = camera_posv * CAMERA_MOVEMENTSPEED * dt / mag.sqrt();
            }

            // TODO: apply rotation to camera vector

            // Camera orientation
            if window.get_key(Key::I) == Action::Press {
                camera_pitch += dt * CAMERA_ROTATIONSPEED;
            }

            if window.get_key(Key::J) == Action::Press {
                camera_yaw += dt * CAMERA_ROTATIONSPEED;
            }

            if window.get_key(Key::K) == Action::Press {
                camera_pitch -= dt * CAMERA_ROTATIONSPEED;
            }

            if window.get_key(Key::L) == Action::Press {
                camera_yaw -= dt * CAMERA_ROTATIONSPEED;
            }

            if mag > 0.0 {
                let movement_rot = Basis3::from_angle_y(Rad(camera_yaw)) * Basis3::from_angle_x(Rad(camera_pitch));

                camera_position += movement_rot.rotate_vector(camera_posv);
            }


            unsafe {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                if let Some(id) = program_id {
                    let program_lifetime = Instant::now().duration_since(program_birth).as_secs_f32();
                    gl::UseProgram(id);
                    set_uniform1f(id, UNIFORM_TIME, program_lifetime);
                    set_uniform1f(id, UNIFORM_CAMPITCH, camera_pitch);
                    set_uniform1f(id, UNIFORM_CAMYAW, camera_yaw);
                    set_uniform3f(id, UNIFORM_CAMPOS, camera_position.x, camera_position.y, camera_position.z);

                    set_uniform1i(id, UNIFORM_SKYBOXSAMPLER, 0);
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_CUBE_MAP, skybox_texobj);

                    gl::BindVertexArray(vao);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
                }
            }

            window.swap_buffers();
            last_render = Instant::now();
        }

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::Size(w, h) => {
                    window_width = w;
                    window_height = h;

                    if let Some(id) = program_id {
                        set_uniform1f(id, 
                            UNIFORM_RATIO, 
                            window_width as f32 / window_height as f32);
                        // supply screenwidth and -height
                        set_uniform2i(id, 
                            UNIFORM_SCREEN_RES,
                            window_width, 
                            window_height);
                    }
                    unsafe {
                        gl::Viewport(0,0,window_width, window_height);
                    }
                }
                glfw::WindowEvent::CursorPos(xpos, ypos) => 
                    println!("Cursor position: ({:?}, {:?})", xpos, ypos),
                _ => {}
            }
        }
    }
}
