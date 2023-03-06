#version 450
#include <world.glsl>
#include <material.glsl>

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 uvOut;

layout(std140, set = 0, binding = 0) uniform WorldUBO {
    World world;
};

layout(push_constant) uniform Constants {
    mat4 viewTransform;
    MaterialInputs mat;
} pc;

void main() {
    uvOut = position;

    vec4 pos = world.proj * pc.viewTransform * vec4(position, 1.0);
    gl_Position = pos.xyww;
}