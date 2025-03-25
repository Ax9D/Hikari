#ifndef FORWARD_PASS_GLOBAL_SET
#define FORWARD_PASS_GLOBAL_SET

#include <world.glsl>
#include <material.glsl>
#include <global_set.glsl>

layout(std140, set = 1, binding = 0) uniform WorldUBO {
    World world;
};
layout(std140, set = 1, binding = 1) readonly buffer InstanceSSBO {
    PerInstanceData perInstanceData[];
};

layout(push_constant) uniform Constants {
    mat4 transform;
    MaterialInputs mat;
} pc;

#endif