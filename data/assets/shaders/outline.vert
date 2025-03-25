#version 450
#include <world.glsl>
#include <material.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tc0;
layout(location = 3) in vec2 tc1;

void main() {
    float thickness = pc.mat.albedo.a;
    mat4 transform = pc.transform;
    vec4 posClip = world.viewProj * transform * vec4(position.xyz, 1.0);
    vec3 normalClip = mat3(world.viewProj) * mat3(transform) * normal;

    gl_Position = vec4(posClip.xyzw);
    gl_Position.xy += (normalize(normalClip.xy) / world.viewportSize) * posClip.w * thickness * 2;
}
