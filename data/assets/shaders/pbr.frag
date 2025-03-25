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
#include <forward_pass_global_set.glsl>
layout(location = 0) in vec3 worldPosition;
layout(location = 1) in vec3 normalFs;
layout(location = 2) in vec2 tc0Fs;
layout(location = 3) in vec2 tc1Fs;
layout(location = 4) in vec3 viewPosition;

layout(location = 0) out vec4 outColor;

layout(std140, set = 2, binding = 0) readonly buffer cascadeRenderInfoSSBO {
    CascadeRenderInfo cascades[];
};
layout(set = 2, binding = 1) uniform sampler2D shadowMap;

#define ALBEDO_OFFSET 0
#define ROUGHNESS_OFFSET 1
#define METALLIC_OFFSET 2
#define NORMAL_OFFSET 3
#define EMISSIVE_OFFSET 4
Surface getSurface() {
    vec2 uv;

    if(pc.mat.uvSet == 0) {
        uv = tc0Fs;
    } else if(pc.mat.uvSet == 1) {
        uv = tc1Fs;
    }
    MaterialInputs mat = pc.mat;

    vec3 normal;
    if(mat.normalIx > 0) {
        normal = getNormalScreen(normalFs, worldPosition, uv, GLOBAL_TEXTURES(mat.normalIx));
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
    MaterialInputs material = pc.mat;
    vec2 uv = surface.uv;

    vec4 albedo = material.albedo;

    if(material.albedoIx > 0) {
        albedo *= texture(GLOBAL_TEXTURES(material.albedoIx), uv);
    }
    
    float perceptualRoughness = clamp(material.roughness, 0.0, 1.0);

    if(material.roughnessIx > 0) {
        perceptualRoughness *= texture(GLOBAL_TEXTURES(material.roughnessIx), uv).g;
    }

    float metallic = clamp(material.metallic, 0.0, 1.0);
    if(material.metallicIx > 0) {
        metallic *= texture(GLOBAL_TEXTURES(material.metallicIx), uv).b;
    }

    vec3 emissive = material.emissive;

    if(material.emissiveIx > 0) {
        emissive *= texture(GLOBAL_TEXTURES(material.emissiveIx), uv).rgb;
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

    vec3 color;

#ifdef LIGHT_MODE_LIT

    PBRMaterialParameters materialParams = computeMaterialParameters(material);
    LightInfo dirLightInfo = getDirectionalLightInfo();

    vec3 direct = BRDF(surface, dirLightInfo, materialParams, GLOBAL_TEXTURES_CUBE(world.envMapIrradianceIx));

    uint cascadeIndex;
    float shadow = dirLightInfo.castShadows == 1 ? getDirectionalShadow(surface, dirLightInfo, cascadeIndex) : 1.0;

    color = direct * shadow;
    
    vec3 indirect = IBL(
                    materialParams, 
                    surface, 
                    world.environmentTransform, 
                    GLOBAL_TEXTURES_CUBE(world.envMapIrradianceIx), 
                    GLOBAL_TEXTURES_CUBE(world.envMapPrefilteredIx), 
                    GLOBAL_TEXTURES(world.BRDFLutIx));

    color+= world.environmentIntensity * indirect;
    color+= material.emissive;
    
    if(world.showCascades == 1) {
        color = length(color) * shadowCascadeDebug(cascadeIndex);
    }
    color*= world.exposure;

#endif

#ifdef LIGHT_MODE_UNLIT
    color = material.albedo.rgb;
#endif
    //color = tonemapUnreal(color * world.exposure);
    //color = color * (1.0f / tonemapACES(vec3(11.2f)));
    color = tonemapFilmic(color);
    //color = pow(color, vec3(1.0/2.2));
    outColor = vec4(color, 1.0);
}