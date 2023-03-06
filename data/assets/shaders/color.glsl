#ifndef COLOR_GLSL
#define COLOR_GLSL

vec3 SRGBtoLINEAR(vec3 srgbIn)
{
    #define MANUAL_SRGB
    #define SRGB_FAST_APPROXIMATION

	#ifdef MANUAL_SRGB
	#ifdef SRGB_FAST_APPROXIMATION
	vec3 linOut = pow(srgbIn,vec3(2.2));
	#else //SRGB_FAST_APPROXIMATION
	vec3 bLess = step(vec3(0.04045),srgbIn);
	vec3 linOut = mix( srgbIn/vec3(12.92), pow((srgbIn+vec3(0.055))/vec3(1.055),vec3(2.4)), bLess );
	#endif //SRGB_FAST_APPROXIMATION
	return linOut;
	#else //MANUAL_SRGB
	return srgbIn;
	#endif //MANUAL_SRGB
}
vec3 LINEARtoSRGB(vec3 linearRGB)
{
    bvec3 cutoff = lessThan(linearRGB, vec3(0.0031308));
    vec3 higher = vec3(1.055)*pow(linearRGB, vec3(1.0/2.4)) - vec3(0.055);
    vec3 lower = linearRGB * vec3(12.92);

    return mix(higher, lower, cutoff);
}

#endif