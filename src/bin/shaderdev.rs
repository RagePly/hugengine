extern crate glfw;
extern crate glad_gl;
extern crate resolution;

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

use hugengine::{define_material, col, transform};
use hugengine::models::{ModelManager, ModelProperty, ModelType};
use hugengine::material::Material;
use hugengine::color::Color;
use hugengine::geospace::Transform;


const UNIFORM_TIME:        &'static str = "uTime";
const UNIFORM_SCREEN_RES:  &'static str = "uScreenResolution";
const UNIFORM_RATIO:       &'static str = "uRatio";
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

fn main() {
    use CompileProgramError::*;
    // load GLFW lib and create a window as well as a opengl-target
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(
        resolution::RES_720P.w as u32, 
        resolution::RES_720P.h as u32, 
        "Shader-dev 0.1", 
        glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    // load opengl function-pointers and fetch current version
    gl::load(|e| glfw.get_proc_address_raw(e) as *const std::os::raw::c_void);

    let (mut majorv, mut minorv) = (0,0);
   
    unsafe {
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut majorv);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut minorv);
        gl::Viewport(0,0,resolution::RES_720P.w as i32,resolution::RES_720P.h as i32);
    }
    println!("using OpenGL version {}.{}", majorv, minorv);

    // vertex-buffer-object, vertex-array-object and ellement-array-buffer-object
    let mut vbo: gl::GLuint = 0;
    let mut vao: gl::GLuint = 0;
    let mut ebo: gl::GLuint = 0;
    let mut index_ssbo: gl::GLuint = 0;
    let mut props_ssbo: gl::GLuint = 0;
    
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

    // Objects
    let mut model_manager = ModelManager::new();

    let _white_ball = model_manager.add_new(ModelProperty {
        t: ModelType::Sphere,
        tf: transform!(),
        color: col!(white),
        material: define_material!(1.0),
    });

    // a red, opaque ball above the sphere
    let _red_box = model_manager.add_new(ModelProperty {
        t: ModelType::Box(1.0,1.0,1.0),
        tf: transform!(0.0,0.5,0.0),
        color: col!(red),
        material: define_material!(0.0),
    });

    let (model_indices, model_properties) = model_manager.create_ss_buffers();


    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut ebo);
        gl::GenBuffers(1, &mut index_ssbo);
        gl::GenBuffers(1, &mut props_ssbo);

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
                                        resolution::RES_720P.get_aspect());
                                    // supply screenwidth and -height
                                    set_uniform2i(new_program_id, 
                                        UNIFORM_SCREEN_RES,
                                        resolution::RES_720P.w as i32, 
                                        resolution::RES_720P.h as i32);

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
        if Instant::now().duration_since(last_render) > Duration::from_millis(33) {
            unsafe {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                if let Some(id) = program_id {
                    let program_lifetime = Instant::now().duration_since(program_birth).as_millis() as f32 / 1000.0;
                    gl::UseProgram(id);
                    set_uniform1f(id, UNIFORM_TIME, program_lifetime);
                    gl::BindVertexArray(vao);
                    gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const c_void);
                }
            }

            window.swap_buffers();
            last_render = Instant::now();
        }

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        glfw::WindowEvent::Size(w, h) => {
            println!("resize: {w}, {h}");
            unsafe {
                gl::Viewport(0,0,w,h);
            }
        }
        _ => {}
    }
}
