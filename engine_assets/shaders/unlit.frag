#version 450
#include <material.glsl>

layout(location = 0) in vec3 worldPosition;
layout(location = 1) in vec3 normalFs;
layout(location = 2) in vec2 tc0Fs;
layout(location = 3) in vec2 tc1Fs;
layout(location = 4) in vec3 viewPosition;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D shadowMap;

layout(set = 1, binding = 0) uniform sampler2D albedoMap;
layout(set = 1, binding = 1) uniform sampler2D roughnessMap;
layout(set = 1, binding = 2) uniform sampler2D metallicMap;
layout(set = 1, binding = 3) uniform sampler2D normalMap;

struct MaterialInputs {
    vec4 albedo;
    float roughness;
    float metallic;
    int albedoUVSet;
    int roughnessUVSet;
    int metallicUVSet;
    int normalUVSet;
};

layout(push_constant) uniform Constants {
    mat4 transform;
    MaterialInputs material;
} pc;

vec4 SRGBtoLINEAR(vec4 srgbIn)
{
	#ifdef MANUAL_SRGB
	#ifdef SRGB_FAST_APPROXIMATION
	vec3 linOut = pow(srgbIn.xyz,vec3(2.2));
	#else //SRGB_FAST_APPROXIMATION
	vec3 bLess = step(vec3(0.04045),srgbIn.xyz);
	vec3 linOut = mix( srgbIn.xyz/vec3(12.92), pow((srgbIn.xyz+vec3(0.055))/vec3(1.055),vec3(2.4)), bLess );
	#endif //SRGB_FAST_APPROXIMATION
	return vec4(linOut,srgbIn.w);;
	#else //MANUAL_SRGB
	return srgbIn;
	#endif //MANUAL_SRGB
}
vec4 LINEARtoSRGB(vec4 linearRGB)
{
    bvec4 cutoff = lessThan(linearRGB, vec4(0.0031308));
    vec4 higher = vec4(1.055)*pow(linearRGB, vec4(1.0/2.4)) - vec4(0.055);
    vec4 lower = linearRGB * vec4(12.92);

    return mix(higher, lower, cutoff);
}

PBRMaterial getMaterial() {
    MaterialInputs material = pc.material;
    
    vec4 albedo = material.albedo;
    if(material.albedoUVSet > -1) {
        albedo *= texture(albedoMap, material.albedoUVSet == 0? tc0Fs: tc1Fs);
    }

    albedo = SRGBtoLINEAR(albedo);
    float perceptualRoughness = material.roughness;
    float metallic = material.metallic;

    if(material.roughnessUVSet > -1) {
        perceptualRoughness *= texture(roughnessMap, material.roughnessUVSet == 0 ? tc0Fs: tc1Fs).g;
    } else {
        perceptualRoughness = clamp(perceptualRoughness, 0.0, 1.0);
    }
    if(material.metallicUVSet > -1) {
        metallic *= texture(metallicMap, material.metallicUVSet == 0 ? tc0Fs: tc1Fs).b;
    } else {
        metallic = clamp(metallic, 0.0, 1.0);
    }

    return PBRMaterial(
        albedo,
        perceptualRoughness,
        metallic
    );
}
void main() {
    PBRMaterial pbrMaterial = getMaterial();
    
    outColor = vec4(pbrMaterial.albedo.rgb, 1.0);
}