#version 450

#include <quad.glsl>

layout(location = 0) out vec2 texCoord;
void main() {
    gl_Position = vec4(positionsCW[gl_VertexIndex], 0.0, 1.0);
    texCoord = texCoordsCW[gl_VertexIndex];
}