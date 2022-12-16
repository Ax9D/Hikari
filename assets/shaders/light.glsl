#ifndef LIGHT_GLSL
#define LIGHT_GLSL

#define N_CASCADES 4

struct ShadowCascade {
    float mapSize;
    float mapTexelSize;
    vec2 atlasUVOffset;
    vec2 atlasSizeRatio;
};
struct CascadeRenderInfo {
    float split;
    float near;
    float far;
    mat4 view;
    mat4 viewProj;
};
struct DirectionalLight {
    float intensity;
    float size;
    float normalBias;
    float maxShadowDistance;
    float shadowFade;
    float shadowSplitLambda;
    vec3 color;
    vec3 direction;
    vec3 upDirection;
    ShadowCascade cascades[N_CASCADES];
};

struct LightInfo {
    float intensity;
    vec3 color;
    vec3 direction;
};

#endif