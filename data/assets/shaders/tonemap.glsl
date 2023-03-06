#ifndef TONEMAP_GLSL
#define TONEMAP_GLSL

vec3 aces(vec3 x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}
// From http://filmicgames.com/archives/75
vec3 Uncharted2Tonemap(vec3 x)
{
	float A = 0.15;
	float B = 0.50;
	float C = 0.10;
	float D = 0.20;
	float E = 0.02;
	float F = 0.30;
	return ((x*(A*x+C*B)+D*E)/(x*(A*x+B)+D*F))-E/F;
}

vec3 tonemapACES(vec3 color)
{
        vec3 outcol = aces(color);
        outcol = outcol * (1.0 / aces(vec3(11.2)));
        return outcol;
}
vec3 tonemapFilmic(vec3 color)
{
    vec3 x = max(vec3(0.0), color - 0.004);
    vec3 result = (x * (6.2 * x + 0.5)) / (x * (6.2 * x + 1.7) + 0.06);
    return result;
}

vec3 tonemapUnreal(vec3 x) {
  return x / (x + 0.155) * 1.019;
}

float tonemapUnreal(float x) {
  return x / (x + 0.155) * 1.019;
}
#endif