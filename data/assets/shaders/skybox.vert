#version 450
#include <world.glsl>
#include <material.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 uvOut;

void main() {
    uvOut = position;

    vec4 pos = world.proj * pc.transform * vec4(position, 1.0);
    gl_Position = pos.xyww;
}