#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tc0;
layout(location = 3) in vec2 tc1;

layout(std140, binding = 0) uniform Camera {
    vec3 cameraPosition;
    mat4 viewProj;
};

layout(location = 0) uniform mat4 transform;

layout(location = 0) out vec3 worldPosition;
layout(location = 1) out vec3 normalFs;
layout(location = 2) out vec2 tc0Fs;
layout(location = 3) out vec2 tc1Fs;



void main() {
    worldPosition = vec3(transform * vec4(position, 1.0));
    normalFs = transpose(inverse(mat3( transform ))) * normal;
    tc0Fs = tc0;
    tc1Fs = tc1;

    gl_Position = viewProj * vec4(worldPosition, 1.0);
}