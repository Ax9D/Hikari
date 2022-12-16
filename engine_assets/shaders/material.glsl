#ifndef MATERIAL_GLSL
#define MATERIAL_GLSL

struct PBRMaterial {
    vec4 albedo;
    float perceptualRoughness;
    float metallic;
};

#endif