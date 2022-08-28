struct World {
    vec3 cameraPosition;
    mat4 view;
    mat4 viewProj;
    float exposure;
    DirectionalLight dirLight;
    uint showCascades;
};