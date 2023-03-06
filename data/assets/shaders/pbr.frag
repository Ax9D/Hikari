#version 450

#include <surface.glsl>
#include <world.glsl>
#include <material.glsl>
#include <brdf.glsl>
#include <ibl.glsl>
#include <shadow.glsl>
#include <pcss.glsl>
#include <tonemap.glsl>
#include <utils.glsl>

layout(location = 0) in vec3 worldPosition;
layout(location = 1) in vec3 normalFs;
layout(location = 2) in vec2 tc0Fs;
layout(location = 3) in vec2 tc1Fs;
layout(location = 4) in vec3 viewPosition;

layout(location = 0) out vec4 outColor;

layout(std140, set = 0, binding = 0) uniform WorldUBO {
    World world;
};
layout(set = 0, binding = 1) uniform sampler2D shadowMap;

layout(std140, set = 0, binding = 2) readonly buffer cascadeRenderInfoSSBO {
    CascadeRenderInfo cascades[];
};

layout(set = 0, binding = 3) uniform samplerCube diffuseIrradianceMap;
layout(set = 0, binding = 4) uniform samplerCube specularPFMap;

layout(set = 1, binding = 0) uniform sampler2D albedoMap;
layout(set = 1, binding = 1) uniform sampler2D roughnessMap;
layout(set = 1, binding = 2) uniform sampler2D metallicMap;
layout(set = 1, binding = 3) uniform sampler2D emissiveMap;
layout(set = 1, binding = 4) uniform sampler2D normalMap;

layout(set = 2, binding = 0) uniform sampler2D brdfLut;

layout(push_constant) uniform Constants {
    mat4 transform;
    MaterialInputs material;
} pc;

#define ALBEDO_OFFSET 0
#define ROUGHNESS_OFFSET 1
#define METALLIC_OFFSET 2
#define NORMAL_OFFSET 3
#define EMISSIVE_OFFSET 4
Surface getSurface() {
    vec2 uv;

    if(pc.material.uvSet == 0) {
        uv = tc0Fs;
    } else if(pc.material.uvSet == 1) {
        uv = tc1Fs;
    }

    vec3 normal;
    if(((pc.material.texturesMask >> NORMAL_OFFSET) & 1) == 1) {
        normal = getNormalScreen(normalFs, worldPosition, uv, normalMap);
    } else
     {
        normal = normalize(normalFs);
    }

    vec3 view = normalize(world.cameraPosition - worldPosition);

    float NdotV = dot(normal, view);
    return Surface(worldPosition, viewPosition, normal, view, uv, NdotV);
}
LightInfo getDirectionalLightInfo() {
    return LightInfo(world.dirLight.intensity, world.dirLight.color, normalize(world.dirLight.direction), world.dirLight.castShadows);
}

PBRMaterial getMaterial(const in Surface surface) {
    MaterialInputs material = pc.material;
    vec2 uv = surface.uv;

    vec4 albedo = material.albedo;
    uint texturesMask = pc.material.texturesMask;

    if(((texturesMask >> ALBEDO_OFFSET) & 1) == 1) {
        albedo *= texture(albedoMap, uv);
    }

    float perceptualRoughness = clamp(material.roughness, 0.0, 1.0);

    if(((texturesMask >> ROUGHNESS_OFFSET) & 1) == 1) {
        perceptualRoughness *= texture(roughnessMap, uv).g;
    }

    float metallic = clamp(material.metallic, 0.0, 1.0);
    if(((texturesMask >> METALLIC_OFFSET) & 1) == 1) {
        metallic *= texture(metallicMap, uv).b;
    }

    vec3 emissive = material.emissive;

    if(((texturesMask >> EMISSIVE_OFFSET) & 1) == 1) {
        emissive *= texture(emissiveMap, uv).rgb;
    }

    return PBRMaterial(
        albedo,
        perceptualRoughness,
        metallic,
        emissive
    );
}
float getDirectionalShadow(Surface surface, LightInfo lightInfo, out uint cascadeIndex) {
    cascadeIndex = 0;

	for(uint i = 0; i < N_CASCADES - 1; ++i) {
		if(surface.viewPosition.z > cascades[i].split) {
			cascadeIndex = i + 1;
		}
	}

    ShadowCascade cascade = world.dirLight.cascades[cascadeIndex];
    ShadowInfo shadowInfo = getShadowInfo(surface,
                                lightInfo,
                                cascades[cascadeIndex].viewProj,
                                cascade.atlasUVOffset,
                                cascade.atlasSizeRatio,
                                cascade.mapTexelSize,
                                world.dirLight.normalBias);
    float intensity = clamp((1.0 - (surface.viewPosition.z / world.dirLight.maxShadowDistance)) / world.dirLight.shadowFade, 0.0, 1.0);

    if(intensity <= 0.0) {
        return 1;
    }

    float shadow = PCFWitness3x3(shadowMap, shadowInfo);
    //float shadow = PCSS(shadowMap, surface, shadowInfo, cascades[cascadeIndex].view, world.cameraNear, world.cameraFar);

    return mix(1.0, shadow, intensity);
}
vec3 shadowCascadeDebug(uint cascadeIndex) {
    switch(cascadeIndex) {
        		case 0:
        			return vec3(1.0f, 0.25f, 0.25f);
        		case 1:
        			return vec3(0.25f, 1.0f, 0.25f);
        		case 2:
        			return vec3(0.25f, 0.25f, 1.0f);
        		case 3:
        			return vec3(1.0f, 1.0f, 0.25f);
    }
}
void main() {
    Surface surface = getSurface();
    PBRMaterial material = getMaterial(surface);

    PBRMaterialParameters materialParams = computeMaterialParameters(material);

    LightInfo dirLightInfo = getDirectionalLightInfo();

    vec3 direct = BRDF(surface, dirLightInfo, materialParams, diffuseIrradianceMap);

    uint cascadeIndex;
    float shadow = dirLightInfo.castShadows == 1 ? getDirectionalShadow(surface, dirLightInfo, cascadeIndex) : 1.0;

    vec3 color = direct * shadow;

    color+= world.environmentIntensity * IBL(materialParams, surface, world.environmentTransform, diffuseIrradianceMap, specularPFMap, brdfLut);
    color+= material.emissive;

    if(world.showCascades == 1) {
        color = length(color) * shadowCascadeDebug(cascadeIndex);
    }
    color*= world.exposure;
    //color = tonemapUnreal(color * world.exposure);
    //color = color * (1.0f / tonemapACES(vec3(11.2f)));
    color = tonemapFilmic(color);
    //color = pow(color, vec3(1.0/2.2));
    outColor = vec4(color, 1.0);
}