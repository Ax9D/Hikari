#ifndef WORLD_GLSL
#define WORLD_GLSL

#include <light.glsl>

struct World {
    vec3 cameraPosition;
    mat4 proj;
    mat4 view;
    mat4 viewProj;
    mat4 environmentTransform;
    float cameraNear;
    float cameraFar;
    vec2 viewportSize;
    float exposure;
    float environmentIntensity;
    uint envMapIx;
    uint envMapIrradianceIx;
    uint envMapPrefilteredIx;
    uint BRDFLutIx;
    DirectionalLight dirLight;
    uint showCascades;
};


struct PerInstanceData {
    mat4 transform;
};

#endif