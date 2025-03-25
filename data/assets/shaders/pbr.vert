#version 450
#include <world.glsl>
#include <material.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tc0;
layout(location = 3) in vec2 tc1;

layout(location = 0) out vec3 worldPosition;
layout(location = 1) out vec3 normalFs;
layout(location = 2) out vec2 tc0Fs;
layout(location = 3) out vec2 tc1Fs;
layout(location = 4) out vec3 viewPosition;

void main() {
    mat4 transform = perInstanceData[gl_InstanceIndex].transform;

    vec4 transPos = transform * vec4(position, 1.0);
    worldPosition = transPos.xyz;
    normalFs = mat3(transpose(inverse(transform))) * normal;

    tc0Fs = tc0;
    tc1Fs = tc1;

    viewPosition = (world.view * transPos).xyz;

    gl_Position = world.viewProj * vec4(worldPosition, 1.0);
}
