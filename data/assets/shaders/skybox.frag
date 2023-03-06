#version 450
#include <tonemap.glsl>

layout(location = 0) in vec3 uv;

layout(set = 0, binding = 1) uniform samplerCube skybox;

layout(location = 0) out vec4 outColor;
void main() {
    vec3 color = texture(skybox, uv).rgb;
    outColor = vec4(tonemapUnreal(color), 1.0);
}