#version 450

#define MAX_STEPS 100
#define MAX_DIST 1000.0
#define SURFACE_DIST 0.01

layout(location=0) out vec4 f_color;


layout(set=0, binding=0)
uniform Uniforms{
    vec3 u_view_position;
    mat4 u_view_proj;
};

float sphereSdf(vec3 p, vec3 spherePos, float radius) {
    return length(p - spherePos) - radius;
}

float boxSdf(vec3 point, vec3 boxPos, vec3 box){
    vec3 p = point + boxPos;
    vec3 q = abs(p) - box;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float GetDist(vec3 point){
    vec3 spherePosition = vec3(0, 1, 6);
    float sphereRadius = 1;
    float sphereDistance = sphereSdf(point, spherePosition, sphereRadius);
    float dPlane = point.y; // Ground plane at 0

    vec3 box = vec3(2,2,2);
    vec3 boxPos = vec3(0,-10,-4);
    float boxDist = boxSdf(point, boxPos, box);
    
    return min(min(sphereDistance, boxDist), dPlane);
}

float RayMarch(vec3 rayOrigin, vec3 rayDirection) {
    float distanceOrigin = 0.0;

    for (int i = 0; i < MAX_STEPS; i++){
        vec3 point = rayOrigin + distanceOrigin * rayDirection;
        float distanceScene = GetDist(point);
        distanceOrigin += distanceScene;
        if (distanceScene < SURFACE_DIST || distanceOrigin > MAX_DIST) {
            break;
        }
    }

    return distanceOrigin;
}

vec3 GetNormal(vec3 point){
    float distance = GetDist(point);
    
    // Get a few points around the point to calculate the normal
    vec2 e = vec2(0.1, 0);
    vec3 normal = distance - vec3(
        GetDist(point - e.xyy),
        GetDist(point - e.yxy),
        GetDist(point - e.yyx)
    );

    return normalize(normal);
}

float GetLight(vec3 point) {
    vec3 lightPosition = vec3(1, 5, 6);
    vec3 light = normalize(lightPosition - point);
    vec3 normal = GetNormal(point);

    float diffuseLight = clamp(dot(normal, light), 0.0, 1.0);

    // Shadow TODO: fix issue with misplaced shadows on SDFs    
    float dist = RayMarch(point + normal * SURFACE_DIST * 2.0, vec3(1.0));
    if (dist < length(lightPosition - point)) {
        diffuseLight *= 0.1;
    }
    

    return diffuseLight;
}

void main(){
    vec2 fragCoord = gl_FragCoord.xy;

    vec3 iResolution = vec3(2560, 1440, 1); // Uniform value

    vec2 uv = (fragCoord - 0.5 * iResolution.xy) / iResolution.y;
    uv.y *= -1; // Not sure why it's flipped
    
    vec3 col = vec3(0);
    vec3 rayOrigin = u_view_position;
    vec3 rayDistance = normalize(vec3(uv.x, uv.y, 1));

    float dist = RayMarch(rayOrigin, rayDistance);
    vec3 point = rayOrigin + rayDistance * dist;
    float diffuseLight = GetLight(point);

    col = vec3(diffuseLight);

    f_color = vec4(col, 1.0);
}