#ifndef FORWARD_PASS_GLOBAL_SET
#define FORWARD_PASS_GLOBAL_SET
#include <world.glsl>
layout(std140, set = 0, binding = 0) uniform WorldUBO {
    World world;
};
layout(set = 0, binding = 1) uniform sampler2D shadowMap;

layout(std140, set = 0, binding = 2) readonly buffer cascadeRenderInfoSSBO {
    CascadeRenderInfo cascades[];
};

layout(set = 0, binding = 3) uniform samplerCube diffuseIrradianceMap;
layout(set = 0, binding = 4) uniform samplerCube specularPFMap;
layout(set = 2, binding = 0) uniform sampler2D brdfLut;

#endif