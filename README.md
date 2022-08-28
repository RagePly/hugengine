# HugEngine
A rust-project for me to learn about graphics rendering, specifically physically based rendering.  

A goal for this project is to design and create a flexible framework for writing shaders, working much like [www.shadertoy.com](https://www.shadertoy.com/).  

Another goal is to write a graphics engine that renders scenes using physically based rendering. This development will guide what features are added to the framework, as it will be used as the base.

The `pbrt` software, described in [the book](https://www.pbrt.org/) by Matt Pharr and others, is a big influence for the algorithms and methods used in this project.

## Installing and Running the software
I don't have any plans at the moment to write documentation nor do I intend on making the installation process streamlined.

That being said, running 
```bash
    cargo run
```
should work if all the below listed dependencies are fulfilled. Any shader-compilation errors will be displayed in the standard output. Currently, you cannot supply shader source-files as arguments, you'll need to change those constants in the code itself.

## Dependencies
### rustc and cargo
rustc and cargo version `1.63.0`.
### GLFW
HugEngine relies on GLFW as a window- and event-handler as well as generating a OpenGL context. It is automatically fetched by cargo from [this repo](https://github.com/bjz/glfw-rs).

### GLAD 2
HugEngine relies on GLAD 2 for loading GL-function pointers. You will have to compile this library yourself and I recommend you use this [this webservice](https://gen.glad.sh/) to generate the source (cargo project). Use the gl-api, __version 4.6__.

You'll need to edit the path in `Cargo.toml`.

### (resolution)
This is a local library that provides some basic screen-resolutions. It will be removed in the future. As for now, there is no public repo and there probably wont ever be one.
