#version 450

#include <world.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) in vec3 position;

void main() {
    mat4 transform = perInstanceData[gl_InstanceIndex].transform;
    vec3 worldPosition = vec3(transform * vec4(position, 1.0));
    gl_Position = world.viewProj * vec4(worldPosition, 1.0);
}