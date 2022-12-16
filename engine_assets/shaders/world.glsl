#ifndef WORLD_GLSL
#define WORLD_GLSL

#include <light.glsl>

struct World {
    vec3 cameraPosition;
    mat4 view;
    mat4 viewProj;
    float cameraNear;
    float cameraFar;
    float exposure;
    DirectionalLight dirLight;
    uint showCascades;
};

#endif