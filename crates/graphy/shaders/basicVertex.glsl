#version 450 core
layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tc;


out vec2 tcOut;

void main() {
        gl_Position = vec4(pos ,1.0);
        tcOut = tc;
}