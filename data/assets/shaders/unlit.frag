#version 450
#include <material.glsl>

layout(location = 0) in vec3 worldPosition;
layout(location = 1) in vec3 normalFs;
layout(location = 2) in vec2 tc0Fs;
layout(location = 3) in vec2 tc1Fs;
layout(location = 4) in vec3 viewPosition;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D shadowMap;

layout(set = 1, binding = 0) uniform sampler2D albedoMap;
layout(set = 1, binding = 1) uniform sampler2D roughnessMap;
layout(set = 1, binding = 2) uniform sampler2D metallicMap;
layout(set = 1, binding = 3) uniform sampler2D emissiveMap;
layout(set = 1, binding = 4) uniform sampler2D normalMap;

layout(push_constant) uniform Constants {
    mat4 transform;
    MaterialInputs material;
} pc;

// vec3 getAlbedo() {
//     vec2 uv = pc.material.uvSet == 0? tc0Fs: tc1Fs;
//     vec4 albedo = pc.material.albedo;
//     if(pc.material.hasAlbedoTex == 1) {
//         albedo *= texture(albedoMap, uv);
//     }

//     return albedo.rgb;
// }
void main() {
    vec3 color = vec3(0.0, 0.5, 0.6);
    outColor = vec4(color, 1.0);
}