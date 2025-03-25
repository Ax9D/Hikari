#ifndef MATERIAL_GLSL
#define MATERIAL_GLSL

struct MaterialInputs {
    vec4 albedo;
    vec3 emissive;
    float roughness;
    float metallic;
    uint uvSet;
    int albedoIx;
    int emissiveIx;
    int roughnessIx;
    int metallicIx;
    int normalIx;
};

struct PBRMaterial {
    vec4 albedo;
    float perceptualRoughness;
    float metallic;
    vec3 emissive;
};

#endif