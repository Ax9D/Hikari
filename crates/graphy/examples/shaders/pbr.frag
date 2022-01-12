#version 450
layout(location = 0) in vec3 worldPosition;
layout(location = 1) in vec3 normalFs;
layout(location = 2) in vec2 tc0Fs;
layout(location = 3) in vec2 tc1Fs;

layout(location = 0) out vec4 color;
//layout(location = 1) out vec4 debugNormal;

layout(std140, set = 0, binding = 0) uniform UBO {
    vec3 cameraPosition;
    mat4 viewProj;
    float exposure;
} ubo;

struct DirectionalLight {
    float intensity;
    vec3 color;
    vec3 direction;
};



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

const float PI = 3.14159265359;
// ----------------------------------------------------------------------------
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    return nom / max(denom, 0.0000001); // prevent divide by zero for roughness=0.0 and NdotH=1.0
}
// ----------------------------------------------------------------------------
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;
    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);
    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(max(1.0 - cosTheta, 0.0), 5.0);
}
vec3 Uncharted2Tonemap(vec3 color)
{
        float A = 0.15;
        float B = 0.50;
        float C = 0.10;
        float D = 0.20;
        float E = 0.02;
        float F = 0.30;
        float W = 11.2;
        return ((color*(A*color+C*B)+D*E)/(color*(A*color+B)+D*F))-E/F;
}
const float gamma = 2.2;
vec3 tonemapLulu(vec3 color)
{
        vec3 outcol = Uncharted2Tonemap(color.rgb * ubo.exposure);
        outcol = outcol * (1.0f / Uncharted2Tonemap(vec3(11.2f)));
        return pow(outcol, vec3(1.0f / gamma));
}
vec3 tonemap(vec3 color) {
    return color/(color + vec3(1.0));
}
vec2 getTc(int set) {
    switch(set) {
        case 0:
            return tc0Fs;
            break;
        case 1:
            return tc1Fs;
            break;
    }
}
vec4 getValueRGBA(sampler2D map, vec4 value, int set) {
    if(set> -1) {
        return texture(map, getTc(set));
    }
    else
        return value;
}
vec3 getValueRGB(sampler2D map, vec3 value, int set) {
    if(set> -1) {
        return texture(map, getTc(set)).rgb;
    }
    else
        return value;
}
vec3 getValueFloat(sampler2D map, float value, int set) {
    if(set> -1) {
        return texture(map, getTc(set)).rgb;
    }
    else
        return vec3(value);
}
vec3 calculateNormal() {
    vec2 tc = getTc(pc.material.normalUVSet);
    vec3 tangentNormal = texture(normalMap, tc).rgb * 2.0 - 1.0;
        vec3 q1 = dFdx(worldPosition);
        vec3 q2 = dFdy(worldPosition);
        vec2 st1 = dFdx(tc);
        vec2 st2 = dFdy(tc);
        vec3 N = normalize(normalFs);
        vec3 T = normalize(q1 * st2.t - q2 * st1.t);
        vec3 B = -normalize(cross(N, T));
        mat3 TBN = mat3(T, B, N);
        return normalize(TBN * tangentNormal);
}
const DirectionalLight dirLight =  {
    10,
    vec3(1.0, 1.0, 1.0),
    vec3(0.0, -1.0, 0.0)
};
void main() {
    Material material = pc.material;
    vec3 albedoValue = getValueRGBA(albedoMap, material.albedo, material.albedoUVSet).rgb;
    float roughnessValue = getValueFloat(roughnessMap, material.roughness, material.roughnessUVSet).g;
    //roughnessValue/=roughness + 0.001;
    float metallicValue = getValueFloat(metallicMap, material.metallic, material.metallicUVSet).b;
    //metallicValue/=metallic + 0.001;
    vec3 normal = material.normalUVSet > -1 ? calculateNormal() : normalize(normalFs);
    vec3 N = normalize(normal);
    vec3 V = normalize(ubo.cameraPosition - worldPosition);
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedoValue, metallicValue);
    vec3 Lo = vec3(0.0);
    vec3 L = -dirLight.direction;
    vec3 H = normalize( V + L);
    vec3 radiance = dirLight.color ;//* dirLight.intensity;
    float NDF = DistributionGGX(N, H, roughnessValue);
    float G = GeometrySmith(N, V, L, roughnessValue);
    vec3 F = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);
    vec3 nominator = NDF * G * F;
    float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
    vec3 specular = nominator / max(denominator, 0.001); //prevent divide by zero for NdotV=0.0 or NdotL=0.0
    // kS is equal to Fresnel
    vec3 kS = F;
    // for energy conservation, the diffuse and specular light can't
    // be above 1.0 (unless the surface emits light); to preserve this
    // relationship the diffuse component (kD) should equal 1.0 - kS.
    vec3 kD = vec3(1.0) - kS;
    // multiply kD by the inverse metalness such that only non-metals
    // have diffuse lighting, or a linear blend if partly metal (pure metals
    // have no diffuse light).
    kD *= 1.0 - metallicValue;
    // scale light by NdotL
    float NdotL = max(dot(N, L), 0.0);
    float illuminance = dirLight.intensity * NdotL;
    Lo += (kD * albedoValue / PI + specular) * radiance * NdotL;
    Lo *= illuminance;
    vec3 ambient = vec3(0.03) * albedoValue;
    vec3 outputColor = ambient + Lo;
    color = vec4( pow(tonemap(outputColor), vec3(1.0/2.2)) , 1.0);
    //debugNormal = vec4(normal, 1.0);
}