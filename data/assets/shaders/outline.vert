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
    MaterialInputs mat;
} pc;

void main() {
    float thickness = pc.mat.albedo.a;

    vec4 posClip = world.viewProj * pc.transform * vec4(position.xyz, 1.0);
    vec3 normalClip = mat3(world.viewProj) * mat3(pc.transform) * normal;

    gl_Position = vec4(posClip.xyzw);
    gl_Position.xy += (normalize(normalClip.xy) / world.viewportSize) * posClip.w * thickness * 2;
}
