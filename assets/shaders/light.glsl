#define MAX_CASCADES 4

struct ShadowCascade {
    float split;
    float near;
    float far;
    mat4 view;
    mat4 viewProj;
};

struct DirectionalLight {
    float intensity;
    float size;
    float constantBiasFactor;
    float normalBiasFactor;
    float maxShadowDistance;
    float shadowFade;
    vec3 color;
    vec3 direction;
    ShadowCascade cascades[MAX_CASCADES];
};

struct LightInfo {
    float intensity;
    vec3 color;
    vec3 direction;
};