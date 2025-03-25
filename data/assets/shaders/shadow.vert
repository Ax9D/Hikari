#version 450

#include <light.glsl>
#include <world.glsl>
#include <forward_pass_global_set.glsl>

layout(location = 0) in vec3 position;

layout(std140, set = 2, binding = 0) readonly buffer cascadeRenderInfoSSBO {
    CascadeRenderInfo cascades[];
};


void main() {
    uint cascadeIx = pc.mat.uvSet;
    mat4 transform = perInstanceData[gl_InstanceIndex].transform;
    gl_Position = cascades[cascadeIx].viewProj * transform * vec4(position, 1.0);
    
    //Pancaking
    //https://www.gamedev.net/forums/topic/639036-shadow-mapping-and-high-up-objects/
    //gl_Position.z = max(gl_Position.z, 0.0);
}