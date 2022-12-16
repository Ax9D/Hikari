#ifndef PCSS_GLSL
#define PCSS_GLSL

#include <shadow.glsl>

const vec2 Poisson[32] = vec2[](
    vec2(-0.975402, -0.0711386),
    vec2(-0.920347, -0.41142),
    vec2(-0.883908, 0.217872),
    vec2(-0.884518, 0.568041),
    vec2(-0.811945, 0.90521),
    vec2(-0.792474, -0.779962),
    vec2(-0.614856, 0.386578),
    vec2(-0.580859, -0.208777),
    vec2(-0.53795, 0.716666),
    vec2(-0.515427, 0.0899991),
    vec2(-0.454634, -0.707938),
    vec2(-0.420942, 0.991272),
    vec2(-0.261147, 0.588488),
    vec2(-0.211219, 0.114841),
    vec2(-0.146336, -0.259194),
    vec2(-0.139439, -0.888668),
    vec2(0.0116886, 0.326395),
    vec2(0.0380566, 0.625477),
    vec2(0.0625935, -0.50853),
    vec2(0.125584, 0.0469069),
    vec2(0.169469, -0.997253),
    vec2(0.320597, 0.291055),
    vec2(0.359172, -0.633717),
    vec2(0.435713, -0.250832),
    vec2(0.507797, -0.916562),
    vec2(0.545763, 0.730216),
    vec2(0.56859, 0.11655),
    vec2(0.743156, -0.505173),
    vec2(0.736442, -0.189734),
    vec2(0.843562, 0.357036),
    vec2(0.865413, 0.763726),
    vec2(0.872005, -0.927)
);

float searchRegionRadiusUV(float z, float near, float lightSizeUV) {
    return lightSizeUV * (z - near) / z;
}
float penumbraRadiusUV(float zReceiver, float zBlocker) {
    return abs(zReceiver - zBlocker) / zBlocker;
}
float projectToLightUV(float z, float near, float penumbraRadius, float lightSizeUV) {
    return penumbraRadius * lightSizeUV * near / z;
}
float zClipToEye(float z, float near, float far) {
    return near + (far - near) * z;
}
void findBlocker(out float avgBlockerDepth, out uint numBlockers, sampler2D shadowMap, ShadowInfo shadowInfo, float searchRegionRadiusUV) {
    avgBlockerDepth = 0.0;
    numBlockers = 0;

    uint blockerSearchSamples = 32;
    for(uint i = 0; i < blockerSearchSamples; i++) {
        vec2 offset = Poisson[i] * searchRegionRadiusUV;

        float blockerDepth = getBlockerDepth(shadowMap, shadowInfo, offset);

        if(shadowInfo.shadowCoord.z < blockerDepth) {
            avgBlockerDepth += blockerDepth;
            numBlockers++;
        }
    }

    avgBlockerDepth /= numBlockers;
}
float PCFPoissonFilter(sampler2D shadowMap, ShadowInfo shadowInfo, float radius) {
    uint PCFSamples = 32;
    float sum = 0.0;
    for(uint i = 0; i < PCFSamples; i++) {
        vec2 offset = Poisson[i] * radius;
        sum+= getShadow(shadowMap, shadowInfo, offset);
    }

    return sum / PCFSamples;
}
float PCSSInner(sampler2D shadowMap, ShadowInfo shadowInfo, float lightViewZ, float near, float far) {
    float lightSizeUV = 0.7;
    float avgBlockerDepth = 0.0;
    uint numBlockers = 0;

    float searchRadiusUV = searchRegionRadiusUV(lightViewZ, near, lightSizeUV);
    findBlocker(avgBlockerDepth, numBlockers, shadowMap, shadowInfo, searchRadiusUV);

    if(numBlockers == 0) {
        return 1.0;
    }

    float avgBlockerDepthEye = zClipToEye(avgBlockerDepth, near, far);
    float penumbraRadius = penumbraRadiusUV(lightViewZ, avgBlockerDepthEye);
    float filterRadius = projectToLightUV(lightViewZ, near, penumbraRadius, lightSizeUV);
    
    return PCFPoissonFilter(shadowMap, shadowInfo, filterRadius);
}
float PCSS(sampler2D shadowMap, Surface surface, ShadowInfo shadowInfo, mat4 lightView, float near, float far) {
    vec4 lightViewPos = lightView * vec4(surface.worldPosition, 1.0);
    //lightViewPos.xyz /= lightViewPos.w;

    return PCSSInner(shadowMap, shadowInfo, lightViewPos.z, near + 0.1, far);
}

#endif