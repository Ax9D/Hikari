struct ShadowInfo {
    vec3 shadowCoord;
    float lightViewDepth;
};

float sampleShadowMap(sampler2D shadowMap, vec2 uv) {
    return texture(shadowMap, vec2(uv.x, 1 - uv.y)).x;
}
float getBlockerDepth(sampler2D shadowMap, vec3 shadowCoord, vec2 offset) {
    vec2 texCoord = shadowCoord.xy;
    float zBlocker = sampleShadowMap(shadowMap, texCoord + offset);
    return zBlocker;
}
ShadowInfo getShadowInfo(Surface surface, LightInfo lightInfo, mat4 lightViewProj, float texelSize, float constantBiasFactor, float normalBiasFactor) {
    mat4 biasMatrix = mat4(
        0.5, 0.0, 0.0, 0.0,
        0.0, 0.5, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.5, 0.5, 0.0, 1.0
    );

    //float slopeBias = clamp(1.0 - dot(surface.normal, -lightInfo.direction), 0.0, 1.0);
    vec3 normalBias = 1 * surface.normal * normalBiasFactor * texelSize * 1.4142136 * 10.0;

    vec4 lightViewDepth = lightViewProj * vec4(surface.worldPosition + normalBias, 1.0);
    
    lightViewDepth.xyz /= lightViewDepth.w;

    vec4 lightSpaceFs = biasMatrix * lightViewDepth;

    vec3 shadowCoord = lightSpaceFs.xyz;

    return ShadowInfo(
        shadowCoord,
        lightViewDepth.z
    );
}
float getShadow(sampler2D shadowMap, vec3 shadowCoord, vec2 offset) {
    float shadow = 1.0;

    float zReceiver = shadowCoord.z;
    if(zReceiver > 1.0 || zReceiver < -1.0) {
        return shadow;
    }
    float zBlocker = getBlockerDepth(shadowMap, shadowCoord, offset);

    if(zBlocker < zReceiver)
        shadow = 0;
    return shadow;
}