#version 450
#include <tonemap.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) in vec3 uv;

layout(location = 0) out vec4 outColor;
void main() {
    vec3 color = texture(GLOBAL_TEXTURES_CUBE(world.envMapIx), uv).rgb;
    outColor = vec4(tonemapUnreal(color), 1.0);
}