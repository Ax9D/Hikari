#version 450

#include <surface.glsl>
#include <world.glsl>
#include <material.glsl>
#include <brdf.glsl>
#include <shadow.glsl>
#include <pcss.glsl>

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

layout(std140, set = 0, binding = 0) uniform WorldUBO {
    World world;
};
layout(std140, set = 0, binding = 2) readonly buffer cascadeRenderInfoSSBO {
    CascadeRenderInfo cascades[];
};
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
// vec3 Uncharted2Tonemap(vec3 color)
// {
//         float A = 0.15;
//         float B = 0.50;
//         float C = 0.10;
//         float D = 0.20;
//         float E = 0.02;
//         float F = 0.30;
//         float W = 11.2;
//         return ((color*(A*color+C*B)+D*E)/(color*(A*color+B)+D*F))-E/F;
// }
const float gamma = 2.2;

vec3 aces(vec3 x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}
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
vec3 tonemapACES(vec3 color)
{
        vec3 outcol = aces(color.rgb * world.exposure);
        outcol = outcol * (1.0f / aces(vec3(11.2f)));
        return pow(outcol, vec3(1.0f / gamma));
}
mat3 cotangentFrame( vec3 N, vec3 p, vec2 uv )
{
    // get edge vectors of the pixel triangle
    vec3 dp1 = dFdx( p );
    vec3 dp2 = dFdy( p );
    vec2 duv1 = dFdx( uv );
    vec2 duv2 = dFdy( uv );
 
    // solve the linear system
    vec3 dp2perp = cross( dp2, N );
    vec3 dp1perp = cross( N, dp1 );
    vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
    vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;
 
    // construct a scale-invariant frame 
    float invmax = inversesqrt( max( dot(T,T), dot(B,B) ) );
    return mat3( T * invmax, B * invmax, N );
}
// vec3 calculateNormal() {
//     vec2 tc = getTc(pc.material.normalUVSet);
//     vec3 normalMapValue = texture(normalMap, tc).rgb;
//     vec3 tangentNormal = normalMapValue * 2.0 - 1.0;
//     vec3 q1 = dFdx(worldPosition);
//     vec3 q2 = dFdy(worldPosition);
//     vec2 st1 = dFdx(tc);
//     vec2 st2 = dFdy(tc);
//     vec3 N = normalize(normalFs);
//     vec3 T = normalize(q1 * st2.t - q2 * st1.t);
//     vec3 B = -normalize(cross(N, T));
//     mat3 TBN = mat3(T, B, N);

//     // vec3 V = ubo.cameraPosition - worldPosition;
//     // vec3 N = normalize(normalFs);
//     // mat3 TBN = cotangent_frame(N, -V, tc);
//     return normalize(TBN * tangentNormal);
// }
vec3 getNormal(MaterialInputs material) {
    vec2 uv = material.normalUVSet == 0 ? tc0Fs : tc1Fs;
    // Perturb normal, see http://www.thetenthplanet.de/archives/1180
	vec3 tangentNormal = texture(normalMap, uv).xyz * 2.0 - 1.0;

	vec3 q1 = dFdx(worldPosition);
	vec3 q2 = dFdy(worldPosition);
	vec2 st1 = dFdx(uv);
	vec2 st2 = dFdy(uv);

	vec3 N = normalize(normalFs);
	vec3 T = normalize(q1 * st2.t - q2 * st1.t);
	vec3 B = -normalize(cross(N, T));
	mat3 TBN = mat3(T, B, N);
    if (!gl_FrontFacing) {
            tangentNormal *= -1.0;
    }
	return normalize(TBN * tangentNormal);
}
vec3 getNormalAlt(MaterialInputs material) {
    vec2 uv = material.normalUVSet == 0 ? tc0Fs : tc1Fs;
    // Perturb normal, see http://www.thetenthplanet.de/archives/1180
	vec3 tangentNormal = texture(normalMap, uv).xyz * 2.0 - 1.0;
	vec3 N = normalize(normalFs);
	mat3 TBN = cotangentFrame(N, worldPosition, uv);
    if (!gl_FrontFacing) {
            tangentNormal *= -1.0;
    }
	return normalize(TBN * tangentNormal);
}

Surface getSurface() {
    vec4 viewPosition = world.viewProj * vec4(worldPosition, 1.0);
    vec3 normal = pc.material.normalUVSet > -1 ? getNormalAlt(pc.material): normalize(normalFs);
    vec3 view = normalize(world.cameraPosition - worldPosition);
    return Surface(worldPosition, viewPosition.xyz, normal, view);
}
LightInfo getDirectionalLightInfo() {
    return LightInfo(world.dirLight.intensity, world.dirLight.color, normalize(world.dirLight.direction));
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
// float getShadowLinear(sampler2D shadowMap, vec3 shadowCoords, vec2 texelSize) {
//     vec2 pixelPos = shadowCoords.xy/texelSize + vec2(0.5);
//     vec2 fracPart = fract(pixelPos);
//     vec2 startTexel = (pixelPos - fracPart) * texelSize;

//     float blTexel = getShadow(shadowMap, shadowCoords, vec2(0.0));
//     float brTexel = getShadow(shadowMap, shadowCoords, vec2(texelSize.x, 0.0));
//     float tlTexel = getShadow(shadowMap, shadowCoords, vec2(0.0, texelSize.y));
//     float trTexel = getShadow(shadowMap, shadowCoords, texelSize);
    
//     float mixA = mix(blTexel, tlTexel, fracPart.x);
//     float mixB = mix(brTexel, trTexel, fracPart.y);

//     return mix(mixA, mixB, fracPart.x);
// }
float getDirectionalShadow(Surface surface, LightInfo lightInfo, uint cascadeIndex) {
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

    float shadow = PCF(shadowMap, shadowInfo);
    //float shadow = PCSS(shadowMap, surface, shadowInfo, cascades[cascadeIndex].view, world.cameraNear, world.cameraFar);

    return mix(1.0, shadow, intensity);
}
void main() {
    PBRMaterial pbrMaterial = getMaterial();
    Surface surface = getSurface();
    LightInfo dirLightInfo = getDirectionalLightInfo();

    BRDFOutput brdfOutput;
    BRDF(surface, dirLightInfo, pbrMaterial, brdfOutput);

    uint cascadeIndex = 0;
	for(uint i = 0; i < N_CASCADES - 1; ++i) {
		if(viewPosition.z < cascades[i].split) {	
			cascadeIndex = i + 1;
		}
	}

    float shadow = getDirectionalShadow(surface, dirLightInfo, cascadeIndex);
    vec3 color = brdfOutput.brdf * shadow + brdfOutput.ambient;

    if(world.showCascades == 1) {
        switch(cascadeIndex) {
        		case 0: 
        			color = length(color) * vec3(1.0f, 0.25f, 0.25f);
        			break;
        		case 1: 
        			color = length(color) * vec3(0.25f, 1.0f, 0.25f);
        			break;
        		case 2: 
        			color = length(color) * vec3(0.25f, 0.25f, 1.0f);
        			break;
        		case 3: 
        			color = length(color) * vec3(1.0f, 1.0f, 0.25f);
        			break;
        }
    }
    
    outColor = vec4(tonemapACES(color), 1.0);
}