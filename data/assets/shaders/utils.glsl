#ifndef UTILS_GLSL
#define UTILS_GLSL

#ifdef HK_FRAGMENT_SHADER
vec3 getNormalScreen(const vec3 normal, const vec3 worldPosition, const vec2 uv, in sampler2D normalMap) {
    // Perturb normal, see http://www.thetenthplanet.de/archives/1180
	vec3 tangentNormal = texture(normalMap, uv).xyz * 2.0 - 1.0;

	vec3 q1 = dFdx(worldPosition);
	vec3 q2 = dFdy(worldPosition);
	vec2 st1 = dFdx(uv);
	vec2 st2 = dFdy(uv);

	vec3 N = normalize(normal);
	vec3 T = normalize(q1 * st2.t - q2 * st1.t);
	vec3 B = -normalize(cross(N, T));
	mat3 TBN = mat3(T, B, N);
    if (!gl_FrontFacing) {
            tangentNormal *= -1.0;
    }
	return normalize(TBN * tangentNormal);
}
#endif
// mat3 cotangentFrame( vec3 N, vec3 p, vec2 uv )
// {
//     // get edge vectors of the pixel triangle
//     vec3 dp1 = dFdx( p );
//     vec3 dp2 = dFdy( p );
//     vec2 duv1 = dFdx( uv );
//     vec2 duv2 = dFdy( uv );

//     // solve the linear system
//     vec3 dp2perp = cross( dp2, N );
//     vec3 dp1perp = cross( N, dp1 );
//     vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;

//     vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;
//     // construct a scale-invariant frame
//     float invmax = inversesqrt( max( dot(T,T), dot(B,B) ) );
//     return mat3( T * invmax, B * invmax, N );
// }
// vec3 getNormalAlt(MaterialInputs material) {
//     vec2 uv = material.normalUVSet == 0 ? tc0Fs : tc1Fs;
//     // Perturb normal, see http://www.thetenthplanet.de/archives/1180
// 	vec3 tangentNormal = texture(normalMap, uv).xyz * 2.0 - 1.0;
// 	vec3 N = normalize(normalFs);
// 	mat3 TBN = cotangentFrame(N, worldPosition, uv);
//     if (!gl_FrontFacing) {
//             tangentNormal *= -1.0;
//     }
// 	return normalize(TBN * tangentNormal);
// }
vec3 cubeNormal(uint faceIndex, vec2 uv) {
    // const vec2 coords = vec2(uv.x, 1.0 - uv.y) * 2.0 - 1.0;
    // switch(faceIndex) {
    //     case 0:
    //         return vec3(1.0, coords.y, -coords.x);
    //     case 1:
    //         return vec3(-1.0, coords.y, coords.x);
    //     case 2:
    //         return vec3(coords.x, 1.0, -coords.y);
    //     case 3:
    //         return vec3(coords.x, -1.0, coords.y);
    //     case 4:
    //         return vec3(coords.x, coords.y, 1.0);
    //     case 5:
    //         return vec3(-coords.x, coords.y, -1.0);
    // }
    // const vec2 coords = vec2(uv.x, 1.0 - uv.y) * 2.0 - 1.0;
    // switch(faceIndex) {
    //     case 0:
    //         return vec3(1.0, coords.y, coords.x);
    //     case 1:
    //         return vec3(-1.0, coords.y, -coords.x);
    //     case 2:
    //         return vec3(-coords.y, 1.0, coords.x);
    //     case 3:
    //         return vec3(coords.y, -1.0, coords.x);
    //     case 4:
    //         return vec3(-coords.x, coords.y, 1.0);
    //     case 5:
    //         return vec3(coords.x, coords.y, -1.0);
    // }
    const vec2 coords = uv * 2.0 - 1.0;
    switch(faceIndex) {
        case 0:
            return vec3(1.0, -coords.y, -coords.x);
        case 1:
            return vec3(-1.0, -coords.y, coords.x);
        case 2:
            return vec3(coords.x, 1.0, coords.y);
        case 3:
            return vec3(coords.x, -1.0, -coords.y);
        case 4:
            return vec3(coords.x, -coords.y, 1.0);
        case 5:
            return vec3(-coords.x, -coords.y, -1.0);
    }
}

vec2 cubeToEquiangular(vec3 n) {
    const vec2 invAtan = vec2(0.1591, 0.3183);
    vec2 uv = vec2(atan(n.z, n.x), asin(n.y));
    uv *= invAtan;
    uv += 0.5;

    uv.y = 1.0 - uv.y;
    return uv;
}
#endif