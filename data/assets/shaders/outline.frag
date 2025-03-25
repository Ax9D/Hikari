#version 450
#include <material.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) out vec4 outColor;

void main() {
    vec3 color = pc.mat.albedo.rgb;
    outColor = vec4(color, 1.0);
}