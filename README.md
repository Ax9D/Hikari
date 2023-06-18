# Hikari
[![Test and Build](https://github.com/Ax9D/Hikari/actions/workflows/ci.yml/badge.svg)](https://github.com/Ax9D/Hikari/actions/workflows/ci.yml)

Hikari is my personal Game Engine that I'm writing in my free time. 

![hikari_render_demo](./demo.png)
![editor_demo](./editor_demo.png)

## Vision
To write a easy to use, modular and performant Game Engine in Rust, and also learn more about writing Game Engines in the process.

### Features
* Physically Based Rendering 
* Image Based Lighting
* Sample Distribution Shadow Mapping
* Multithreaded Asset Loader
* Shader Hot Reloading
* Powerful Render Graph API powered by Vulkan
* Compute Shader support
* ECS Architecture
* WYSIWYG Editor for loading and editing Scenes (imgui)

### Current Goals
- [ ] Editor and QoL Improvements
- [ ] Add scripting functionality (Lua)
- [ ] Animation System
- [ ] Bindless Render Resources
- [ ] Global Illumination (DDGI)

## Build
To build Hikari, you'll need to have Rust installed, along with the VulkanSDK.

To build everything in the workspace, run:
```rust
cargo build --all
```

# Project Structure

Hikari is divided into a number of smaller crates found inside inside the `crates` folder. All of the functionality in these crates is reexported by the main `hikari` crate.

There are also two additional crates `hikari_editor` and `hikari_cli`.

You can run the editor from the project root: 
```rust
cargo run -p hikari_editor
```
Note: By default the debug builds are compiled with optimizations

## Thanks
Here is a list of projects that I'd like to thank which have heavily inspired the development of Hikari:

* [Hazel](https://github.com/TheCherno/Hazel) - Hazel Engine
* [Bevy](https://github.com/bevyengine/bevy) - A refreshingly simple data-driven game engine built in Rust
* [kajiya](https://github.com/EmbarkStudios/kajiya) - Experimental real-time global illumination renderer
* [Granite](https://github.com/Themaister/Granite)
* [Godot](https://github.com/godotengine/godot) - Multi-platform 2D and 3D game engine
* [egui-gizmo](https://github.com/urholaukkarinen/egui-gizmo) - 3d transformation gizmo built on top of the egui library
