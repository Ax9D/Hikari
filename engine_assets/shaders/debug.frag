#version 450 core 
layout(location = 0) in vec2 texCoords;


layout(set = 0, binding = 0) uniform sampler2D depthMap;
layout(set = 0, binding = 1) uniform sampler2D directionalShadowMap;

layout(location = 0) out vec4 depthMapDebug;
layout(location = 1) out vec4 directionalShadowMapDebug;

float linearize_Z(float depth , float zNear , float zFar){
    depth = depth * 2 - 1;
    return (2*zNear * zFar ) / (zFar + zNear - depth*(zFar -zNear)) ;
}

void main() {
    float linear_depth = linearize_Z(texture(depthMap, texCoords).r, 0.1, 1000.0);
    depthMapDebug = vec4(vec3(linear_depth), 1.0);
    directionalShadowMapDebug = vec4(texture(directionalShadowMap, texCoords).rrr, 1.0);
}