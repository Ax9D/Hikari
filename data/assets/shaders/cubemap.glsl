#version 450 

layout(location = 0) in vec3 pos;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
} pc;

layout(location = 0) out vec3 uvOut;

void main() {
    uvOut = pos;
    gl_Position = pc.mvp * vec4(pos, 1.0);
}