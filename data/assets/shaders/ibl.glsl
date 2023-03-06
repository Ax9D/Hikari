#ifndef IBL_GLSL
#define IBL_GLSL

#include <brdf.glsl>

float radicalInverse_VdC(uint bits)
{
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits) * 2.3283064365386963e-10; // / 0x100000000
}
vec2 hammersley(uint i, uint N)
{
    return vec2(float(i)/float(N), radicalInverse_VdC(i));
}

vec3 importanceSampleGGX(vec2 u, vec3 n, float alphaRoughness)
{
    const float a2 = alphaRoughness * alphaRoughness;

    const float phi = 2.0 * PI * u.x;
    const float cosTheta = sqrt((1.0 - u.y) / (1.0 + (a2 - 1.0) * u.y));
    const float sinTheta = sqrt(1.0 - cosTheta*cosTheta);

    // from spherical coordinates to cartesian coordinates
    vec3 h;
    h.x = cos(phi) * sinTheta;
    h.y = sin(phi) * sinTheta;
    h.z = cosTheta;

    // from tangent-space vector to world-space sample vector
    const vec3 up        = abs(n.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    const vec3 tangent   = normalize(cross(up, n));
    const vec3 bitangent = cross(n, tangent);

    const vec3 sampleVec = tangent * h.x + bitangent * h.y + n * h.z;
    return normalize(sampleVec);
}
vec3 importanceSampleCosDir(vec2 u, vec3 n) {
    // from tangent-space vector to world-space sample vector
    const vec3 up        = abs(n.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    const vec3 tangent   = normalize(cross(up, n));
    const vec3 bitangent = cross(n, tangent);

    const float r = sqrt(u.x);
    const float phi = 2.0 * PI * u.y;

    // from spherical coordinates to cartesian coordinates
    vec3 l;
    l.x = r * cos(phi);
    l.y = r * sin(phi);
    l.z = sqrt(max(0.0, 1 - u.x));

    const vec3 sampleVec = tangent * l.x + bitangent * l.y + n * l.z;
    return normalize(sampleVec);
}

// vec3 diffuseIrradianceConvolve(const vec3 n, in samplerCube envMap) {
//     vec3 up = abs(n.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
//     const vec3 right = normalize(cross(up, n));
//     up = normalize(cross(n, right));

//     const float TWO_PI = PI * 2.0;
//     const float HALF_PI = PI * 0.5;

//     uint nSamples = 0;
//     const float delta = 0.01;
//     vec3 irradiance = vec3(0.0);
//     for(float phi = 0.0; phi < TWO_PI; phi += delta) {
//         for(float theta = 0.0; theta < HALF_PI; theta += delta) {
//             const vec3 tangent = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
//             const vec3 texSample = tangent.x * right + tangent.y * up + tangent.z * n;
//             // vec3 tempVec = cos(phi) * right + sin(phi) * up;
// 			// vec3 sampleVector = cos(theta) * N + sin(theta) * tempVec;

//             vec3 colorSample = textureLod(envMap, texSample, 0).rgb;
//             irradiance += colorSample * cos(theta) * sin(theta);
//             nSamples++;
//         }
//     }
//     irradiance = PI * irradiance / float(nSamples);

//     return irradiance;
// }
vec3 diffuseIrradianceConvolve(const vec3 n, in samplerCube envMap) {
    const vec3 v = n;

    const uint nSamples = 1024;
    vec3 color = vec3(0.0);
    float total = 0.0;

    float envMapSize = float(textureSize(envMap, 0).x);

    for(uint i = 0; i < nSamples; i++) {
        const vec2 u = hammersley(i, nSamples);
        const vec3 l  = importanceSampleCosDir(u, n);

        const float NdotL = clamp(dot(n, l), 0.0, 1.0);

        if(NdotL > 0.0) {
            float pdf = NdotL / PI;

        	// Slid angle of current smple
			float omegaS = 1.0 / (float(nSamples) * pdf);
			// Solid angle of 1 pixel across all cube faces
			float omegaP = 4.0 * PI / (6.0 * envMapSize * envMapSize);

            float mipLevel = max(0.5 * log2(omegaS / omegaP) + 1.0, 0.0);
            color += textureLod(envMap, l, mipLevel).rgb * NdotL;
            total += NdotL;
        }
    }

    return color / total;
}
vec3 specularPrefilterConvolve(vec3 n, float roughness, in samplerCube envMap) {
    const vec3 v = n;
    const float alphaRoughness = roughness * roughness;

    const uint nSamples = 1024;
    vec3 color = vec3(0.0);
    float total = 0.0;

    float envMapSize = float(textureSize(envMap, 0).x);

    for(uint i = 0; i < nSamples; i++) {
        const vec2 u = hammersley(i, nSamples);
        const vec3 h  = importanceSampleGGX(u, n, alphaRoughness);
        const vec3 l  = normalize(2.0 * dot(v, h) * h - v);

        const float NdotL = clamp(dot(n, l), 0.0, 1.0);

        if(NdotL > 0.0) {
            float NdotH = clamp(dot(n, h), 0.0, 1.0);
            float VdotH = clamp(dot(v, h), 0.0, 1.0);

            float D   = D_GGXDividePI(NdotH, alphaRoughness);
            float pdf = (D * NdotH / (4.0 * VdotH)) + 0.0001;

        	// Slid angle of current smple
			float omegaS = 1.0 / (float(nSamples) * pdf);
			// Solid angle of 1 pixel across all cube faces
			float omegaP = 4.0 * PI / (6.0 * envMapSize * envMapSize);

            float mipLevel = roughness == 0.0 ? 0.0 : max(0.5 * log2(omegaS / omegaP) + 1.0, 0.0);
            color += textureLod(envMap, l, mipLevel).rgb * NdotL;
            total += NdotL;
        }
    }

    return color / total;
}
//Takes a instead of (a + 1)
float GeometrySchlickGGX_IBL(float NdotV, float alphaRoughness)
{
    float a = alphaRoughness;
    float k = (a * a) / 2.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
float V_Smith_IBL(float NdotV, float NdotL, float alphaRoughness) {
    float ggx2  = GeometrySchlickGGX_IBL(NdotV, alphaRoughness);
    float ggx1  = GeometrySchlickGGX_IBL(NdotL, alphaRoughness);

    return ggx1 * ggx2;
}
vec2 integrateBRDF(float NdotV, float roughness) {
    const float alphaRoughness = roughness * roughness;
    vec3 v;
    v.x = sqrt(1.0 - NdotV * NdotV); // sin
    v.y = 0.0;
    v.z = NdotV; // cos

    // N points straight upwards for this integration
    const vec3 n = vec3(0.0, 0.0, 1.0);

    float A = 0.0;
    float B = 0.0;
    const uint nSamples = 1024;

    for (uint i = 0; i < nSamples; i++) {
        const vec2 u = hammersley(i, nSamples);
        const vec3 h  = importanceSampleGGX(u, n, alphaRoughness);
        const vec3 l  = reflect(-v, h);

        //const float NdotL = clamp(dot(n, l), 0.0, 1.0);
        //const float NdotH = clamp(dot(n, h), 0.0, 1.0);
        //const float VdotH = clamp(dot(v, h), 0.0, 1.0);

        const float NdotL = max(0.0, l.z);
        const float NdotH = max(0.0, h.z);
        const float VdotH = max(0.0, dot(v, h));

        if(NdotL > 0.0) {
            //float V = V_SmithGGXCorrelated(NdotV, NdotL, alphaRoughness) * VdotH * NdotL / NdotH;
            const float V = V_Smith_IBL(NdotV, NdotL, roughness);
            const float V_Vis = (V * VdotH) / (NdotH * NdotV);
            const float Fc = pow(1.0 - VdotH, 5.0);

            A += (1.0 - Fc) * V_Vis;
            B += Fc * V_Vis;
        }
    }

    return vec2(A, B) / vec2(nSamples);
}

// Calculation of the lighting contribution from an optional Image Based Light source.
// Precomputed Environment Maps are required uniform inputs and are computed as outlined in [1].
// See our README.md on Environment Maps [3] for additional discussion.
vec3 IBL(const in PBRMaterialParameters material, const in Surface surface, in mat4 transform, in samplerCube diffuseIrradianceMap, in samplerCube specularPFMap, in sampler2D brdfLUT)
{
    const vec3 v = surface.view;
    const vec3 n = surface.normal;
	const vec3 r = normalize(reflect(-v, n));

    const vec3 transformedN = (transform * vec4(n, 1.0)).xyz;
    const vec3 transformedR = (transform * vec4(r, 1.0)).xyz;

    const float NdotV = clamp(surface.NdotV, 0.001, 1.0);

    const vec3 F = F_SchlickRoughness(material.f0, NdotV, material.perceptualRoughness);

    const vec3 kS = F;
    const vec3 kD = (1.0 - kS);

    const vec3 irradiance = textureLod(diffuseIrradianceMap, transformedN, 0).rgb;
    const vec3 diffuse = kD * irradiance * material.diffuseColor;

    const float mipCount = 9.0; // resolution of 512x512
    const float lod = (material.perceptualRoughness * (mipCount - 1.0));
    const vec3 specularPrefiltered = textureLod(specularPFMap, transformedR, lod).rgb;

    const vec2 brdfUV = vec2(NdotV, material.perceptualRoughness);
    const vec3 brdf = texture(brdfLUT, brdfUV).rgb;

    const vec3 specular = specularPrefiltered * (kS * brdf.x + brdf.y);

    return diffuse + specular;
    //return specular;
    //return brdf;
}

#endif