#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tc0;
layout(location = 3) in vec2 tc1;

layout(std140, set = 0, binding = 0) uniform UBO {
    vec3 cameraPosition;
    mat4 viewProj;
    float exposure;
} ubo;


layout(location = 0) out vec3 worldPosition;
layout(location = 1) out vec3 normalFs;
layout(location = 2) out vec2 tc0Fs;
layout(location = 3) out vec2 tc1Fs;

struct Material {
    vec4 albedo;
    float roughness;
    float metallic;
    int albedoUVSet;
    int roughnessUVSet;
    int metallicUVSet;
    int normalUVSet;
};

layout(push_constant) uniform Constants {
    mat4 transform;
    Material material;
} pc;


void main() {
    worldPosition = vec3(pc.transform * vec4(position, 1.0));
    normalFs = transpose(inverse(mat3( pc.transform ))) * normal;
    tc0Fs = tc0;
    tc1Fs = tc1;

    gl_Position = ubo.viewProj * vec4(worldPosition, 1.0);
}
