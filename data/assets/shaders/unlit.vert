#version 450
#include <world.glsl>
#include <material.glsl>

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tc0;
layout(location = 3) in vec2 tc1;

layout(std140, set = 0, binding = 0) uniform WorldUBO {
    World world;
};

layout(push_constant) uniform Constants {
    mat4 transform;
    MaterialInputs _material;
} pc;

void main() {
    vec4 transPos = pc.transform * vec4(position, 1.0);
    vec3 worldPosition = vec3(transPos);

    gl_Position = world.viewProj * vec4(worldPosition, 1.0);
}
