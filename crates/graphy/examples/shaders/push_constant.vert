#version 450 core

layout(location = 0) in vec2 position;

layout(location = 0) out vec3 outColor;

layout(push_constant) uniform constants {
    vec4 color;
    vec2 position;
} pushConstants;

void main() {
    gl_Position = vec4(position + pushConstants.position, 1.0, 1.0);
    outColor = pushConstants.color.rgb;
}