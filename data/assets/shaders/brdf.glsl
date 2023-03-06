#ifndef BRDF_GLSL
#define BRDF_GLSL

#include<surface.glsl>
#include<light.glsl>
#include<material.glsl>

const float PI = 3.14159265359;

// Basic Lambertian diffuse
// Implementation from Lambert's Photometria https://archive.org/details/lambertsphotome00lambgoog
// See also [1], Equation 1
float Fd_Lambert() {
    return 1.0 / PI;
}
// Schlick 1994, "An Inexpensive BRDF Model for Physically-Based Rendering"
vec3 F_Schlick(vec3 f0, vec3 f90, float VoH) {
    return f0 + (f90 - f0) * pow(1.0 - VoH, 5.0);
}

vec3 F_Schlick(vec3 f0, float VoH) {
    const float f = pow(1.0 - VoH, 5.0);
    return f + f0 * (1.0 - f);
}

vec3 F_SchlickRoughness(vec3 F0, float cosTheta, float roughness)
{
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(1.0 - cosTheta, 5.0);
}

float F_Schlick(float f0, float f90, float VoH) {
    return f0 + (f90 - f0) * pow(1.0 - VoH, 5.0);
}


float _D_GGX(float NoH, float alphaRoughness) {
	const float roughnessSq = alphaRoughness * alphaRoughness;
	const float f = NoH * NoH * (roughnessSq - 1.0) + 1.0;
	return roughnessSq / (f * f);
}
float D_GGXDividePI(float NoH, float alphaRoughness) {
	return _D_GGX(NoH, alphaRoughness) / PI;
}

float V_SmithGGXCorrelated(float NoV, float NoL, float alphaRoughness) {
    float a2 = alphaRoughness * alphaRoughness;
    float GGXV = NoL * sqrt(NoV * NoV * (1.0 - a2) + a2);
    float GGXL = NoV * sqrt(NoL * NoL * (1.0 - a2) + a2);
    return 0.5 / (GGXV + GGXL);
}

float V_Smith(float NoV, float NoL, float alphaRoughness) {
    const float a2 = alphaRoughness * alphaRoughness;
    const float V_Smith_V = NoV + sqrt(NoV * (NoV - NoV * a2) + a2);
    const float V_Smith_L = NoL + sqrt(NoL * (NoL - NoL * a2) + a2);
    return 1.0 / (V_Smith_V * V_Smith_L);
}

struct PBRMaterialParameters {
	float perceptualRoughness;// roughness value, as authored by the model creator (input to shader)
	float alphaRoughness;// roughness mapped to a more linear change in the roughness (proposed by [2])
	float metallic;// metallic value at the surface
	vec3 f0;// full reflectance color (normal incidence angle)
	vec3 f90;// reflectance color at grazing angle
	vec3 albedo;// material albedo
	vec3 diffuseColor;// color contribution from diffuse lighting
};

PBRMaterialParameters computeMaterialParameters(const in PBRMaterial inputMat) {
    const vec4 baseColor = inputMat.albedo;
    const float perceptualRoughness = inputMat.perceptualRoughness;
    const float metallic = inputMat.metallic;

    const vec3 f0 = vec3(0.04);

    vec3 diffuseColor;
    diffuseColor = baseColor.rgb * (vec3(1.0) - f0);
    diffuseColor *= 1.0 - metallic;
    const vec3 specularColor = mix(f0, baseColor.rgb, metallic);

    const float alphaRoughness = perceptualRoughness * perceptualRoughness;
    const float reflectance = max(max(specularColor.r, specularColor.g), specularColor.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
	// For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
	const float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
	const vec3 specularEnvironmentR0 = specularColor;
	const vec3 specularEnvironmentR90 = vec3(reflectance90);

	return PBRMaterialParameters(
			perceptualRoughness,
			alphaRoughness,
			metallic,
			specularEnvironmentR0,
			specularEnvironmentR90,
			baseColor.rgb,
			diffuseColor
	);
}

vec3 BRDF(const Surface surface, const LightInfo lightInfo, const PBRMaterialParameters material, in samplerCube diffuseIrradianceMap) {
	vec3 n = surface.normal;
	vec3 v = surface.view;
	vec3 l = -lightInfo.direction;

    vec3 h = normalize(l+v);

    float NdotL = clamp(dot(n, l), 0.001, 1.0);
	float NdotV = clamp(abs(surface.NdotV), 0.001, 1.0);
	float NdotH = clamp(dot(n, h), 0.0, 1.0);
	float LdotH = clamp(dot(l, h), 0.0, 1.0);
	float VdotH = clamp(dot(v, h), 0.0, 1.0);

	const vec3 F = F_Schlick(material.f0, VdotH);
	const float V = V_SmithGGXCorrelated(NdotL, NdotV, material.alphaRoughness);
	const float D = D_GGXDividePI(NdotH, material.alphaRoughness);

    const vec3 kS = F;
    vec3 kD = (1.0 - kS);

	vec3 diffuse = kD *  material.diffuseColor * Fd_Lambert();
	vec3 specular = F * V * D;
	//vec3 specular = F * V * D / (4.0 * NdotV * NdotL);

	// Obtain final intensity as reflectance (BRDF) scaled by the energy of the light (cosine law)
    vec3 reflectance = NdotL * (diffuse + specular);

    return reflectance * lightInfo.color * lightInfo.intensity;
}

#endif