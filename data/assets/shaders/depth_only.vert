#version 450

#include <world.glsl>

layout(location = 0) in vec3 position;

layout(push_constant) uniform PushConstants {
    mat4 transform;
} pc;

layout(std140, set = 0, binding = 0) uniform WorldUBO {
    World world;
};

void main() {
    vec3 worldPosition = vec3(pc.transform * vec4(position, 1.0));
    gl_Position = world.viewProj * vec4(worldPosition, 1.0);
}