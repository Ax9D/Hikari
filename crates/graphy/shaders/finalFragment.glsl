#version 420 core
layout(location = 0) out vec4 SCREEN_COLOR;

layout(binding = 3) uniform sampler2D offscreen;
layout(binding = 1) uniform sampler2D depthTest;

uniform float bright;
in vec2 tcOut;

void main() {
    SCREEN_COLOR = vec4(texture(offscreen, tcOut).rgb, 1.0);
}