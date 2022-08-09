#version 450
layout(location = 0) in vec3 worldPosition;
layout(location = 1) in vec3 normalFs;
layout(location = 2) in vec2 tc0Fs;
layout(location = 3) in vec2 tc1Fs;

layout(location = 0) out vec4 outColor;

struct DirectionalLight {
    float intensity;
    vec3 color;
    vec3 direction;
};

layout(std140, set = 0, binding = 0) uniform UBO {
    vec3 cameraPosition;
    mat4 viewProj;
    float exposure;

    DirectionalLight dirLight;
} ubo;


layout(set = 1, binding = 0) uniform sampler2D albedoMap;
layout(set = 1, binding = 1) uniform sampler2D roughnessMap;
layout(set = 1, binding = 2) uniform sampler2D metallicMap;
layout(set = 1, binding = 3) uniform sampler2D normalMap;

struct Material {
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
    Material material;
} pc;
struct PBRInfo
{
	float NdotL;                  // cos angle between normal and light direction
	float NdotV;                  // cos angle between normal and view direction
	float NdotH;                  // cos angle between normal and half vector
	float LdotH;                  // cos angle between light direction and half vector
	float VdotH;                  // cos angle between view direction and half vector
	float perceptualRoughness;    // roughness value, as authored by the model creator (input to shader)
	float metalness;              // metallic value at the surface
	vec3 reflectance0;            // full reflectance color (normal incidence angle)
	vec3 reflectance90;           // reflectance color at grazing angle
	float alphaRoughness;         // roughness mapped to a more linear change in the roughness (proposed by [2])
	vec3 diffuseColor;            // color contribution from diffuse lighting
	vec3 specularColor;           // color contribution from specular lighting
};

const float PI = 3.14159265359;
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
vec3 tonemapACES(vec3 color)
{
        vec3 outcol = aces(color.rgb * ubo.exposure);
        outcol = outcol * (1.0f / aces(vec3(11.2f)));
        return pow(outcol, vec3(1.0f / gamma));
}

mat3 cotangent_frame( vec3 N, vec3 p, vec2 uv )
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
vec3 getNormal() {
    Material material = pc.material;
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
vec3 getNormalAlt() {
    Material material = pc.material;
    vec2 uv = material.normalUVSet == 0 ? tc0Fs : tc1Fs;
    // Perturb normal, see http://www.thetenthplanet.de/archives/1180
	vec3 tangentNormal = texture(normalMap, uv).xyz * 2.0 - 1.0;
	vec3 N = normalize(normalFs);
	mat3 TBN = cotangent_frame(N, worldPosition, uv);
    if (!gl_FrontFacing) {
            tangentNormal *= -1.0;
    }
	return normalize(TBN * tangentNormal);
}
// Basic Lambertian diffuse
// Implementation from Lambert's Photometria https://archive.org/details/lambertsphotome00lambgoog
// See also [1], Equation 1
vec3 diffuse(PBRInfo pbrInputs) {
    return pbrInputs.diffuseColor / PI;
}

// The following equation models the Fresnel reflectance term of the spec equation (aka F())
// Implementation of fresnel from [4], Equation 15
vec3 specularReflection(PBRInfo pbrInputs)
{
	return pbrInputs.reflectance0 + (pbrInputs.reflectance90 - pbrInputs.reflectance0) * pow(clamp(1.0 - pbrInputs.VdotH, 0.0, 1.0), 5.0);
}

float V_SmithGGXCorrelated(float NoV, float NoL, float roughness) {
    float a2 = roughness * roughness;
    float GGXV = NoL * sqrt(NoV * NoV * (1.0 - a2) + a2);
    float GGXL = NoV * sqrt(NoL * NoL * (1.0 - a2) + a2);
    return 0.5 / (GGXV + GGXL);
}

// This calculates the specular geometric attenuation (aka G()),
// where rougher material will reflect less light back to the viewer.
// This implementation is based on [1] Equation 4, and we adopt their modifications to
// alphaRoughness as input as originally proposed in [2].
float geometricOcclusion(PBRInfo pbrInputs)
{
	float NdotL = pbrInputs.NdotL;
	float NdotV = pbrInputs.NdotV;
	float r = pbrInputs.alphaRoughness;

    float a2 = r * r;

	float attenuationL = 2.0 * NdotL / (NdotL + sqrt(a2 + (1.0 - a2) * (NdotL * NdotL)));
	float attenuationV = 2.0 * NdotV / (NdotV + sqrt(a2 + (1.0 - a2) * (NdotV * NdotV)));
	return attenuationL * attenuationV;

    //return V_SmithGGXCorrelated(NdotV, NdotL, r);
}


// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
float microfacetDistribution(PBRInfo pbrInputs)
{
	float roughnessSq = pbrInputs.alphaRoughness * pbrInputs.alphaRoughness;
	float f = (pbrInputs.NdotH * roughnessSq - pbrInputs.NdotH) * pbrInputs.NdotH + 1.0;
	return roughnessSq / (PI * f * f);
}

vec3 EnvBRDFApprox(vec3 f0, float perceptualRoughness, float NdotV) {
    vec4 c0 = vec4(-1.0, -0.0275, -0.572, 0.022);
    vec4 c1 = vec4(1.0, 0.0425, 1.04, -0.04);
    vec4 r = perceptualRoughness * c0 + c1;
    float a004 = min(r.x * r.x, exp2(-9.28 * NdotV)) * r.x + r.y;
    vec2 AB = vec2(-1.04, 1.04) * a004 + r.zw;
    return f0 * AB.x + AB.y;
}

vec3 BRDF(vec3 n, vec3 v, vec3 l, vec3 lightColor, float intensity) {
    float perceptualRoughness;
    float metallic;
    vec3 diffuseColor;
    vec4 baseColor;
    
    vec3 f0 = vec3(0.04);

    Material material = pc.material;
    baseColor = material.albedo;
    if(material.albedoUVSet > -1) {
        baseColor *= texture(albedoMap, material.albedoUVSet == 0? tc0Fs: tc1Fs);
    }

    perceptualRoughness = material.roughness;
    metallic = material.metallic;

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

    diffuseColor = baseColor.rgb * (vec3(1.0) - f0);
    diffuseColor *= 1.0 - metallic;

    float alphaRoughness = perceptualRoughness * perceptualRoughness;
    
    vec3 specularColor = mix(f0, baseColor.rgb, metallic);
    float reflectance = max(max(specularColor.r, specularColor.g), specularColor.b);
    
    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
	// For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
	float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
	vec3 specularEnvironmentR0 = specularColor.rgb;
	vec3 specularEnvironmentR90 = vec3(1.0, 1.0, 1.0) * reflectance90;

    vec3 h = normalize(l+v);
    vec3 reflection = normalize(reflect(-v, n));
    reflection.y *= -1.0f;

    float NdotL = clamp(dot(n, l), 0.001, 1.0);
	float NdotV = clamp(abs(dot(n, v)), 0.001, 1.0);
	float NdotH = clamp(dot(n, h), 0.0, 1.0);
	float LdotH = clamp(dot(l, h), 0.0, 1.0);
	float VdotH = clamp(dot(v, h), 0.0, 1.0);

	PBRInfo pbrInputs = PBRInfo(
		NdotL,
		NdotV,
		NdotH,
		LdotH,
		VdotH,
		perceptualRoughness,
		metallic,
		specularEnvironmentR0,
		specularEnvironmentR90,
		alphaRoughness,
		diffuseColor,
		specularColor
	);
    // Calculate the shading terms for the microfacet specular shading model
	vec3 F = specularReflection(pbrInputs);
	float G = geometricOcclusion(pbrInputs);
	float D = microfacetDistribution(pbrInputs);

    // Calculation of analytical lighting contribution
    vec3 diffuseContrib = (1.0 - F) * diffuse(pbrInputs);
    vec3 specContrib = F * G * D / (4.0 * NdotL * NdotV);
    
	// Obtain final intensity as reflectance (BRDF) scaled by the energy of the light (cosine law)
    vec3 brdf = NdotL * (diffuseContrib + specContrib);

    vec3 diffuseAmbient = EnvBRDFApprox(diffuseColor, 1.0, NdotV);
    vec3 specularAmbient = EnvBRDFApprox(f0, perceptualRoughness, NdotV);
    float ambientFactor = 0.1;
    return brdf * lightColor * intensity + (diffuseAmbient + specularAmbient) * ambientFactor;
}
void main() {
    Material material = pc.material;

    vec3 n = material.normalUVSet > -1 ? getNormalAlt(): normalize(normalFs);
    
    vec3 v = normalize(ubo.cameraPosition - worldPosition);
    vec3 l = normalize(-ubo.dirLight.direction);

    vec3 color = BRDF(n, v, l, ubo.dirLight.color, ubo.dirLight.intensity);

    outColor = vec4(tonemapACES(color), 1.0);
}