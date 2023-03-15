#version 450
#include <material.glsl>
layout(location = 0) out vec4 outColor;

layout(push_constant) uniform Constants {
    layout(offset = 0) mat4 transform;
    MaterialInputs _material;
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