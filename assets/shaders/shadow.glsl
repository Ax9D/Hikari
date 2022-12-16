#ifndef SHADOW_GLSL
#define SHADOW_GLSL

struct ShadowInfo {
    vec3 shadowCoord;
    float lightViewDepth;
    vec2 atlasUVOffset;
    vec2 atlasSizeRatio;
    float mapTexelSize;
};

float sampleShadowMap(sampler2D shadowMap, vec2 uv, vec2 atlasUVOffset, vec2 atlasSizeRatio) {
    if(uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0)
        return 1.0;

    uv = uv * atlasSizeRatio + atlasUVOffset;

    vec2 atlasCoords = vec2(uv.x, 1.0 - uv.y);
    return texture(shadowMap, atlasCoords).x;
}
float getBlockerDepth(sampler2D shadowMap, ShadowInfo shadowInfo, vec2 uvOffset) {
    vec2 texCoord = shadowInfo.shadowCoord.xy;
    float zBlocker = sampleShadowMap(shadowMap, texCoord + uvOffset, shadowInfo.atlasUVOffset, shadowInfo.atlasSizeRatio);
    return zBlocker;
}
vec2 ComputeReceiverPlaneDepthBias(vec3 texCoordDX, vec3 texCoordDY)
{
    vec2 biasUV;
    biasUV.x = texCoordDY.y * texCoordDX.z - texCoordDX.y * texCoordDY.z;
    biasUV.y = texCoordDX.x * texCoordDY.z - texCoordDY.x * texCoordDX.z;
    biasUV *= 1.0 / ((texCoordDX.x * texCoordDY.y) - (texCoordDX.y * texCoordDY.x));
    return biasUV;  
}
ShadowInfo getShadowInfo(Surface surface, LightInfo lightInfo, mat4 lightViewProj, vec2 atlasUVOffset, vec2 atlasSizeRatio, float mapTexelSize, float normalBiasFactor) {
    mat4 biasMatrix = mat4(
        0.5, 0.0, 0.0, 0.0,
        0.0, 0.5, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.5, 0.5, 0.0, 1.0
    );

    //float slopeBias = clamp(1.0 - dot(surface.normal, -lightInfo.direction), 0.0, 1.0);
    vec3 normalBias = surface.normal * normalBiasFactor * mapTexelSize * 2 * 10.0;

    vec4 lightViewDepth = lightViewProj * vec4(surface.worldPosition + normalBias, 1.0);
    
    lightViewDepth.xyz /= lightViewDepth.w;
    vec4 lightSpaceFs = biasMatrix * lightViewDepth;

    vec3 shadowCoord = lightSpaceFs.xyz;

    return ShadowInfo(
        shadowCoord,
        lightViewDepth.z,
        atlasUVOffset,
        atlasSizeRatio,
        mapTexelSize
    );
}
float getShadow(sampler2D shadowMap, ShadowInfo shadowInfo, vec2 offset) {
    float shadow = 1.0;

    float zReceiver = shadowInfo.shadowCoord.z;
    // if(zReceiver > 1.0 || zReceiver < -1.0) {
    //     return shadow;
    // }
    float zBlocker = getBlockerDepth(shadowMap, shadowInfo, offset);

    if(zBlocker < zReceiver)
        shadow = 0;
    return shadow;
}
float getShadowRecvDepth(sampler2D shadowMap, ShadowInfo shadowInfo, vec2 baseUV, vec2 offset, vec2 receiverPlaneDepthBias) {
    float zReceiver = shadowInfo.shadowCoord.z ;//+ dot(offset * shadowInfo.mapTexelSize, receiverPlaneDepthBias);

    shadowInfo.shadowCoord.xy = baseUV;

    float zBlocker = getBlockerDepth(shadowMap, shadowInfo, offset * shadowInfo.mapTexelSize);

    float shadow = 1.0;
    if(zBlocker < zReceiver)
        shadow = 0;
    return shadow;
}
float PCF(sampler2D shadowMap, ShadowInfo shadowInfo) {
    vec2 texelSize = vec2(shadowInfo.mapTexelSize);

    float shadowFactor = 0.0;
    int count = 0;
    int range = 2;



const vec2 Poisson[64] = vec2[](
    vec2(-0.5119625, -0.4827938),
    vec2(-0.2171264, -0.4768726),
    vec2(-0.7552931, -0.2426507),
    vec2(-0.7136765, -0.4496614),
    vec2(-0.5938849, -0.6895654),
    vec2(-0.3148003, -0.7047654),
    vec2(-0.42215, -0.2024607),
    vec2(-0.9466816, -0.2014508),
    vec2(-0.8409063, -0.03465778),
    vec2(-0.6517572, -0.07476326),
    vec2(-0.1041822, -0.02521214),
    vec2(-0.3042712, -0.02195431),
    vec2(-0.5082307, 0.1079806),
    vec2(-0.08429877, -0.2316298),
    vec2(-0.9879128, 0.1113683),
    vec2(-0.3859636, 0.3363545),
    vec2(-0.1925334, 0.1787288),
    vec2(0.003256182, 0.138135),
    vec2(-0.8706837, 0.3010679),
    vec2(-0.6982038, 0.1904326),
    vec2(0.1975043, 0.2221317),
    vec2(0.1507788, 0.4204168),
    vec2(0.3514056, 0.09865579),
    vec2(0.1558783, -0.08460935),
    vec2(-0.0684978, 0.4461993),
    vec2(0.3780522, 0.3478679),
    vec2(0.3956799, -0.1469177),
    vec2(0.5838975, 0.1054943),
    vec2(0.6155105, 0.3245716),
    vec2(0.3928624, -0.4417621),
    vec2(0.1749884, -0.4202175),
    vec2(0.6813727, -0.2424808),
    vec2(-0.6707711, 0.4912741),
    vec2(0.0005130528, -0.8058334),
    vec2(0.02703013, -0.6010728),
    vec2(-0.1658188, -0.9695674),
    vec2(0.4060591, -0.7100726),
    vec2(0.7713396, -0.4713659),
    vec2(0.573212, -0.51544),
    vec2(-0.3448896, -0.9046497),
    vec2(0.1268544, -0.9874692),
    vec2(0.7418533, -0.6667366),
    vec2(0.3492522, 0.5924662),
    vec2(0.5679897, 0.5343465),
    vec2(0.5663417, 0.7708698),
    vec2(0.7375497, 0.6691415),
    vec2(0.2271994, -0.6163502),
    vec2(0.2312844, 0.8725659),
    vec2(0.4216993, 0.9002838),
    vec2(0.4262091, -0.9013284),
    vec2(0.2001408, -0.808381),
    vec2(0.149394, 0.6650763),
    vec2(-0.09640376, 0.9843736),
    vec2(0.7682328, -0.07273844),
    vec2(0.04146584, 0.8313184),
    vec2(0.9705266, -0.1143304),
    vec2(0.9670017, 0.1293385),
    vec2(0.9015037, -0.3306949),
    vec2(-0.5085648, 0.7534177),
    vec2(0.9055501, 0.3758393),
    vec2(0.7599946, 0.1809109),
    vec2(-0.2483695, 0.7942952),
    vec2(-0.4241052, 0.5581087),
    vec2(-0.1020106, 0.6724468)
);
    // for(int x = -range; x <= range; x++) {
    //     for(int y = -range; y <= range; y++) {
    //         vec2 offset = texelSize * (vec2(x, y) + Poisson[(x * 2 * range + y) % 32]);

    //         shadowFactor += getShadow(shadowMap, shadowInfo, offset);
    //         count++;
    //     }
    // }

    for(uint i = 0 ; i < 32; i++) {
        shadowFactor += getShadow(shadowMap, shadowInfo, vec2(Poisson[i] * texelSize));
        count++;
    }
    return shadowFactor / count;
}
float PCFWitness3x3(sampler2D shadowMap, ShadowInfo shadowInfo) {
    vec2 texelSize = vec2(shadowInfo.mapTexelSize);

    float sum = 0.0;

    //vec3 shadowPosDX = dFdxFine(shadowInfo.shadowCoord);
    //vec3 shadowPosDY = dFdyFine(shadowInfo.shadowCoord);

    vec2 receiverPlaneDepthBias = vec2(0.0);//ComputeReceiverPlaneDepthBias(shadowPosDX, shadowPosDY);
    float depth = shadowInfo.shadowCoord.z;

    // Static depth biasing to make up for incorrect fractional sampling on the shadow map grid
    //float fractionalSamplingError = 2.0 * dot(vec2(1.0) * texelSize, abs(receiverPlaneDepthBias));
    //depth -= min(fractionalSamplingError, 0.01);

    vec2 uv = shadowInfo.shadowCoord.xy * (1.0 / texelSize);

    vec2 baseUV;
    baseUV.x = floor(uv.x + 0.5);
    baseUV.y = floor(uv.y + 0.5);

    float s = (uv.x + 0.5 - baseUV.x);
    float t = (uv.y + 0.5 - baseUV.y);

    baseUV -= vec2(0.5, 0.5);
    baseUV *= texelSize;

    float uw0 = (3 - 2 * s);
    float uw1 = (1 + 2 * s);

    float u0 = (2 - s) / uw0 - 1;
    float u1 = s / uw1 + 1;

    float vw0 = (3 - 2 * t);
    float vw1 = (1 + 2 * t);

    float v0 = (2 - t) / vw0 - 1;
    float v1 = t / vw1 + 1;

    sum += uw0 * vw0 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u0, v0), receiverPlaneDepthBias);
    sum += uw1 * vw0 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u1, v0), receiverPlaneDepthBias);
    sum += uw0 * vw1 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u0, v1), receiverPlaneDepthBias);
    sum += uw1 * vw1 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u1, v1), receiverPlaneDepthBias);

    return sum * 1.0 / 16.0;
}
float PCFWitness5x5(sampler2D shadowMap, ShadowInfo shadowInfo) {
    vec2 texelSize = vec2(shadowInfo.mapTexelSize);

    float sum = 0.0;

    //vec3 shadowPosDX = dFdxFine(shadowInfo.shadowCoord);
    //vec3 shadowPosDY = dFdyFine(shadowInfo.shadowCoord);

    vec2 receiverPlaneDepthBias = vec2(0.0); //ComputeReceiverPlaneDepthBias(shadowPosDX, shadowPosDY);
    float depth = shadowInfo.shadowCoord.z;

    // Static depth biasing to make up for incorrect fractional sampling on the shadow map grid
    //float fractionalSamplingError = 2.0 * dot(vec2(1.0) * texelSize, abs(receiverPlaneDepthBias));
    //depth -= min(fractionalSamplingError, 0.01);

    vec2 uv = shadowInfo.shadowCoord.xy * (1.0 / texelSize);

    vec2 baseUV;
    baseUV.x = floor(uv.x + 0.5);
    baseUV.y = floor(uv.y + 0.5);

    float s = (uv.x + 0.5 - baseUV.x);
    float t = (uv.y + 0.5 - baseUV.y);

    baseUV -= vec2(0.5, 0.5);
    baseUV *= texelSize;

    float uw0 = (4 - 3 * s);
    float uw1 = 7;
    float uw2 = (1 + 3 * s);

    float u0 = (3 - 2 * s) / uw0 - 2;
    float u1 = (3 + s) / uw1;
    float u2 = s / uw2 + 2;

    float vw0 = (4 - 3 * t);
    float vw1 = 7;
    float vw2 = (1 + 3 * t);

    float v0 = (3 - 2 * t) / vw0 - 2;
    float v1 = (3 + t) / vw1;
    float v2 = t / vw2 + 2;

    sum += uw0 * vw0 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u0, v0), receiverPlaneDepthBias);
    sum += uw1 * vw0 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u1, v0), receiverPlaneDepthBias);
    sum += uw2 * vw0 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u2, v0), receiverPlaneDepthBias);

    sum += uw0 * vw1 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u0, v1), receiverPlaneDepthBias);
    sum += uw1 * vw1 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u1, v1), receiverPlaneDepthBias);
    sum += uw2 * vw1 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u2, v1), receiverPlaneDepthBias);

    sum += uw0 * vw2 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u0, v2), receiverPlaneDepthBias);
    sum += uw1 * vw2 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u1, v2), receiverPlaneDepthBias);
    sum += uw2 * vw2 * getShadowRecvDepth(shadowMap, shadowInfo, baseUV, vec2(u2, v2), receiverPlaneDepthBias);

    return sum * 1.0 / 144;
}

#endif