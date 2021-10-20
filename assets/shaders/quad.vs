#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tc;

layout(location = 1) out vec2 tex_coord;

void main() {
    gl_Position = vec4(position, 1.0);
    tex_coord = tc;
}