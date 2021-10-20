#version 420 core
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tc;

out vec3 normalOut;
out vec3 fragPos;
out vec2 tcOut;

uniform float scale;
void main() {
        gl_Position = vec4(pos * scale,1.0);
        
        tcOut = tc;
        normalOut = normal;
        fragPos = pos*scale;
}