#ifdef HK_FRAGMENT_SHADER
#include <material.glsl>
#extension GL_EXT_nonuniform_qualifier : enable

layout(set = 0, binding = 0) uniform sampler2D g_Textures[];
layout(set = 0, binding = 0) uniform sampler3D g_Textures_3d[];
layout(set = 0, binding = 0) uniform samplerCube g_Textures_Cube[];

#define GLOBAL_TEXTURES(index) g_Textures[nonuniformEXT(index)]
#define GLOBAL_TEXTURES_3D(index) g_Textures_3d[nonuniformEXT(index)]
#define GLOBAL_TEXTURES_CUBE(index) g_Textures_Cube[nonuniformEXT(index)]

layout(set = 0, binding = 1, rgba8) uniform image2D g_Images_rgba8[];
layout(set = 0, binding = 1, rgba16f) uniform image2D g_Images_rgba16f[];
layout(set = 0, binding = 1, rgba32f) uniform image2D g_Images_rgba32f[];

#define GLOBAL_IMAGES_RGBA8(index) g_Images_rgba8[nonuniformEXT(index)]
#define GLOBAL_IMAGES_RGBA16F(index) g_Images_rgba16f[nonuniformEXT(index)]
#define GLOBAL_IMAGES_RGBA32F(index) g_Images_rgba32f[nonuniformEXT(index)]

layout(set = 0, binding = 2, std140) buffer MaterialsBuffer {
    MaterialInputs material;
} g_Materials[];

#endif