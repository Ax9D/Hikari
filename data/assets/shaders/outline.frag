#version 450
#include <material.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) out vec4 outColor;

layout(push_constant) uniform Constants {
    mat4 transform;
    MaterialInputs mat;
} pc;

void main() {
    vec3 color = pc.mat.albedo.rgb;
    outColor = vec4(color, 1.0);
}