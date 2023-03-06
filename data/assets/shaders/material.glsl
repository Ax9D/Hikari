#ifndef MATERIAL_GLSL
#define MATERIAL_GLSL

struct MaterialInputs {
    vec4 albedo;
    vec3 emissive;
    float roughness;
    float metallic;
    uint uvSet;
    uint texturesMask;
    //uint hasAlbedoTex;
    //uint hasRoughnessTex;
    //uint hasMetallicTex;
    //uint hasEmissiveTex;
    //uint hasNormalTex;
};

struct PBRMaterial {
    vec4 albedo;
    float perceptualRoughness;
    float metallic;
    vec3 emissive;
};

#endif