#version 450

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 3) uniform sampler2D color;

void main() {
    outColor = vec4(fragColor.rgb, 1.0);
}