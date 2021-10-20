#version 450 core
layout(location = 7) out vec3 offscreen;

in vec3 fragPos;
in vec2 tcOut;
in vec3 normalOut;

const vec3 LIGHT_POS = vec3(1.0,1.0,1.0);

void main() {
    vec3 lightDir = normalize(LIGHT_POS - -fragPos);

    float brightness = max(dot(normalOut, lightDir), 0.0);
    
    offscreen = vec3(brightness);
}
        