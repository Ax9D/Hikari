#version 450

#include <light.glsl>
#include <world.glsl>

layout(location = 0) in vec3 position;

layout(push_constant) uniform PushConstants {
    mat4 transform;
    uint cascade_ix;
} pc;

layout(std140, set = 0, binding = 0) uniform WorldUBO {
    World world;
};
layout(std140, set = 0, binding = 1) readonly buffer cascadeRenderInfoSSBO {
    CascadeRenderInfo cascades[];
};

void main() {
    gl_Position = cascades[pc.cascade_ix].viewProj * pc.transform * vec4(position, 1.0);
    
    //Pancaking
    //https://www.gamedev.net/forums/topic/639036-shadow-mapping-and-high-up-objects/
    //gl_Position.z = max(gl_Position.z, 0.0);
}