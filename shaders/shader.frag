#version 460 core

#define PI 3.141592653589793
#define SQRT2 1.4142135623730951
#define INV_SQRT2 0.7071067811865475
#define MAX_REFLECTIONS 4
#define BUMB_AMOUNT 0.00001


in vec2 ScreenPos;
out vec4 FragColor;

uniform float uTime;                // time since start, sec
uniform float uRatio;               // width:height ratio
uniform ivec2 uScreenResolution;    // screen-resolution in pixels, int
uniform float uCamPitch;            // Camera pitch in rad
uniform float uCamYaw;              // Camera yaw in rad
uniform vec3  uCamPos;              // Camera position

// NOTE: this binding is statically typed in the shaderdevprogram.
layout(std430, binding = 4) buffer ModelIndex 
{
    // [object0_type, object0_index, object1_type, object1_index,...]
    int uModelIndex[];
};

layout(std430, binding = 5) buffer ModelProperties 
{
    float uModelProps[];
};

// SHAPING FUNCTIONS

// scales and translates x such that the range [0, 1] becomes [a, b]. 
// Will invert if a > b and cause the range afterwards to be [b, a].
float squash(in float x, in float a, in float b) {
    return x * (b - a) + a;
}

// COLOR FUNCTIONS

float deg2rad(in float deg) {
    return PI * deg / 180.0;
}

vec4 rgba(in int r, in int g, in int b, in int a) {
    return vec4(
            float(r) / 255.0,
            float(g) / 255.0,
            float(b) / 255.0,
            float(a) / 255.0);
}
vec4 rgb(in int r, in int g, in int b) {
    return rgba(r,g,b,255);
}

vec4 blend(in vec4 col1, in vec4 col2, in float factor) {
    float f = clamp(factor, 0.0, 1.0);
    return col1 * f + col2 * (1.0 - f);
}

// TRANSFORM MATRICES
mat4 translation(const in vec3 t) {
    return mat4(1.0f, 0.0f, 0.0f, 0.0f,
                0.0f, 1.0f, 0.0f, 0.0f,
                0.0f, 0.0f, 1.0f, 0.0f,
                 t.x,  t.y,  t.z, 1.0f);
}

mat4 rotate_x(const in float theta) {
    float s = sin(theta);
    float c = cos(theta);
    return mat4(1.0f, 0.0f, 0.0f, 0.0f,
                0.0f,    c,    s, 0.0f,
                0.0f,   -s,    c, 0.0f,
                0.0f, 0.0f, 0.0f, 1.0f);
}

mat4 rotate_y(const in float theta) {
    float s = sin(theta);
    float c = cos(theta);
    return mat4(   c, 0.0f,   -s, 0.0f,
                0.0f, 1.0f, 0.0f, 0.0f,
                   s, 0.0f,    c, 0.0f,
                0.0f, 0.0f, 0.0f, 1.0f);
}

mat4 rotate_z(const in float theta) {
    float s = sin(theta);
    float c = cos(theta);
    return mat4(   c,    s, 0.0f, 0.0f,
                  -s,    c, 0.0f, 0.0f,
                0.0f, 0.0f, 1.0f, 0.0f,
                0.0f, 0.0f, 0.0f, 1.0f);
}

mat4 euler_transform(const in float h, const in float p, const in float r) {
    return rotate_z(r) * rotate_x(p) * rotate_y(h);
}

// quaternions
#define quat vec4
quat qmult(in quat q, in quat r) {
    vec3 q_v = q.xyz;
    vec3 r_v = r.xyz;
    return quat(cross(q_v, r_v) + r.w * q_v + q.w * r_v, q.w * r.w - dot(q_v, r_v));
}

quat qconj(in quat q) {
    return quat(-q.xyz, q.w);
}

float qnorm(in quat q) {
    return length(q);
}

float qnorm2(in quat q) {
    return dot(q, q);
}

quat qinv(in quat q) {
    return qconj(q) / qnorm2(q);
}

quat qunit(in vec3 u, in float theta) {
    return quat(sin(theta) * normalize(u), cos(theta));
}

quat qrotate(in quat q, in vec4 p) {
    return qmult(q, qmult(p, qinv(q)));
}

// source: real-time rendering 4ed, eq4.46 (c4.3.2, p80)
mat4 quat2mat4(in quat q) {
    return mat4(1.0 - 2.0 * (q.y*q.y + q.z*q.z),   2.0 * (q.x * q.y + q.w * q.z),   2.0 * (q.x * q.z - q.w * q.y), 0.0,
                  2.0 * (q.x * q.y - q.w * q.z), 1.0 - 2.0 * (q.x*q.x + q.z*q.z),   2.0 * (q.y * q.z + q.w * q.x), 0.0,
                  2.0 * (q.x * q.z + q.w * q.y),   2.0 * (q.y * q.z - q.w * q.x), 1.0 - 2.0 * (q.x*q.x + q.y*q.y), 0.0,
                                            0.0,                             0.0,                             0.0, 1.0);

}

struct EulerRotation {
    float head;
    float pitch;
    float roll;
};

struct Properties {
    // Transform
    vec3 position;
    float scale;
    mat4 rotation;

    // Color properties
    vec3 color;

    // Material properties
    float reflectance;
};

/*  
 *  Retrieving data from the properties buffer
 */

// fetch vec4 of the position for the object `i`
vec4 get_position(int i)
{
    return vec4(uModelProps[i],
                uModelProps[i+1],
                uModelProps[i+2],
                1.0
           );
}

// fetch vec3 of the position for the object `i`
vec3 get_position3(int i)
{
    return vec3(uModelProps[i],
                uModelProps[i+1],
                uModelProps[i+2]
           );
}

float get_scale(int i) {
    return uModelProps[i+3];
}

vec3 get_color3(int i) {
    return vec3(uModelProps[i+7], uModelProps[i+8], uModelProps[i+9]);
}

vec4 get_color(int i) {
    return vec4(uModelProps[i+7], uModelProps[i+8], uModelProps[i+9], 1.0);
}

mat4 get_rotation(int i) {
    return euler_transform(
            uModelProps[i+4],
            uModelProps[i+5],
            uModelProps[i+6]);
}

float get_reflectance(int i) {
    return uModelProps[i+10];
}


// populate the Properties-struct `prop` for the object `i`
#define fetch_props(prop, i)                \
    prop.position = get_position3(i);       \
    prop.scale = get_scale(i);              \
    prop.rotation = get_rotation(i);        \
    prop.color = vec3(                      \
            uModelProps[i+7],               \
            uModelProps[i+8],               \
            uModelProps[i+9]                \
            );                              \
    prop.reflectance = uModelProps[i+10];   

#define SPHERE_ID 0
#define BOX_ID 1
#define PLANE_ID 2



// TODO: implement robus version, check sec3.9.4
// Solves a quadratic equation, not accepting complex solutions.
// Returns:
//    true, if a solution is found
//    false, if a solution is not found
bool quadratic_solve(
        const in float a, 
        const in float b, 
        const in float c, 
        out float x1, 
        out float x2
        ) 
{
    float det = b*b - 4.0 * a * c;
    if (det >= 0) {
        float sdet = sqrt(det);
        x1 = (-b + sdet) / (2.0 * a);    
        x2 = (-b - sdet) / (2.0 * a);
        return true;
    } else {
        return false;
    }
}

bool intersection_sphere(
        in vec3 ray_o,
        in vec3 ray_d,
        in vec3 s_o,
        in float s_r,
        in float t_max,
        out float t_entry,  // ingoing intersection
        out float t_exit    // outgoing intersection
        )
{
    // source: physically based rendering, s3.2, p135
    vec3 o = ray_o - s_o;
    float a = ray_d.x * ray_d.x + ray_d.y * ray_d.y + ray_d.z * ray_d.z;
    float b = 2.0 * (ray_d.x * o.x + ray_d.y * o.y + ray_d.z * o.z); 
    float c = o.x * o.x + o.y * o.y + o.z * o.z - s_r * s_r;

    if (quadratic_solve(a, b, c, t_exit, t_entry)) {
        return 0.0 < t_entry && t_entry < t_max;
    } else {
        return false;
    }
}

vec3 normal_sphere(in vec3 sphere, in vec3 point) {
    return normalize(point - sphere);
}

bool draw_sphere(
        in int i,
        in vec3 ray_o,
        in vec3 ray_d,
        in float t_max,
        out float t_intersect,
        out vec3 normal
        )
{
    // fetch data
    vec3 sphere_pos = get_position3(i);
    float t_entry, t_exit;
    if (intersection_sphere(
                ray_o,
                ray_d,
                sphere_pos,
                get_scale(i),
                t_max,
                t_entry,
                t_exit
                )
            )
    {
        t_intersect = t_entry;
        normal = normal_sphere(sphere_pos, ray_o + t_intersect * ray_d);
        return true;
    } else {
        return false;
    }
}

vec4 draw_box(int i, vec2 pos) {
    Properties props;
    fetch_props(props, i);
    vec3 dims = vec3(uModelProps[i+11], uModelProps[i+12], uModelProps[i+13]);
    if (
        props.position.x <= pos.x && pos.x <= props.position.x + dims.x &&
        props.position.y <= pos.y && pos.y <= props.position.y + dims.y
    ) 
    {
        return vec4(props.color, 1.0);
    } else {
        return vec4(0.0,0.0,0.0,0.0);
    }
}

// TODO: implement a more robust version
bool intersection_plane(
        in vec3 ray_o,
        in vec3 ray_d,
        in vec3 plane_p,
        in vec3 plane_n,
        in float t_max,
        out float t
        )
{
    float a = dot(plane_n, ray_o - plane_p);
    float b = dot(plane_n, ray_d);
    if (b != 0.0) {
        t = -a / b;
        return 0.0 < t && t < t_max;
    }
    return false;
}

bool draw_plane(
        in int i,
        in vec3 ray_o,
        in vec3 ray_d,
        in float t_max,
        out float t_intersect,
        out vec3 normal
        )
{
    mat4 rotm = get_rotation(i);

    vec4 temp = rotm * vec4(0.0, 1.0, 0.0, 1.0); // TODO: change this to match rotation-transform
    normal = normalize(temp.xyz);
    
    return intersection_plane(
            ray_o, 
            ray_d, 
            get_position3(i),
            normal,
            t_max,
            t_intersect);
}

void create_ray(in float fov_deg, out vec3 ray_o, out vec3 ray_d) {
    float half_fov_rad = deg2rad(fov_deg / 2.0);
    float z = uRatio / tan(half_fov_rad);


    ray_o = vec3(uRatio * ScreenPos.x, ScreenPos.y, 0.0);
    ray_d = normalize(ray_o - vec3(0.0,0.0,z));

    // create rotation matrix, pitch then yaw
    mat4 rot = rotate_y(uCamYaw) * rotate_x(uCamPitch);
    mat4 trans = translation(uCamPos);

    ray_d = (rot * vec4(ray_d, 1.0)).xyz;
    ray_o = (trans * rot * vec4(ray_o, 1.0)).xyz;
}

struct ReflectionData {
    int model_type;  // type of object
    int model_index; // index into properties of object this reflection represents
    vec3 normal;     // normal-vector of the intersection point
    vec3 ray_dir;    // direction of ray
    vec3 ray_orig;   // ray origin
    float dist;      // distance from the last source, t_value
};

void main()
{
    vec3 ray_o, ray_d;
    create_ray(90.0, ray_o, ray_d);

    float t_max = 1000.0;
    int reflections = 0;
    
    ReflectionData reflect_stack[MAX_REFLECTIONS];

    for (; reflections < MAX_REFLECTIONS; reflections++) {
        float t_smallest = t_max;
        int i_smallest = 0;
        vec3 reflection_normal;

        // loop through every object
        bool intersection_found = false;
        for (int i = 0; i < uModelIndex.length() / 2; i++) 
        {
            vec3 temp_reflection;
            int model_type = uModelIndex[2 * i];
            int prop_index = uModelIndex[2 * i + 1];
            float t_intersect;
            bool flag = false;

            switch (model_type) {
                case SPHERE_ID: // sphere
                    flag = draw_sphere(prop_index, 
                                ray_o, 
                                ray_d, 
                                t_max,
                                t_intersect,
                                temp_reflection);
                    break;
                case PLANE_ID: // plane
                    flag = draw_plane(prop_index, 
                                ray_o, 
                                ray_d, 
                                t_max,
                                t_intersect,
                                temp_reflection);
                    break;
                default:
                    break;
            }

            // any intersection happened
            if (flag) {
                if (t_intersect < t_smallest) {
                    t_smallest = t_intersect;
                    i_smallest = i;
                    reflection_normal = temp_reflection;
                }
                intersection_found = true;
            }
        }

        if (intersection_found) {
            reflect_stack[reflections].model_type  = uModelIndex[i_smallest * 2];
            reflect_stack[reflections].model_index = uModelIndex[i_smallest * 2 + 1];
            reflect_stack[reflections].normal      = reflection_normal;
            reflect_stack[reflections].ray_dir     = ray_d;
            reflect_stack[reflections].ray_orig    = ray_o;
            reflect_stack[reflections].dist        = t_smallest;

            // apply reflection
            ray_o = ray_o + ray_d * t_smallest + reflection_normal * BUMB_AMOUNT;
            ray_d = reflect(ray_d, reflection_normal);
        } else {
            break;
        }
    }

    // apply sky-color from the last vector direction
    float latitude = dot(ray_d, vec3(0.0, 1.0, 0.0));
    vec4 sky_color = rgb(85, 170, 224);
    vec4 sunset_color = rgb(246, 109, 73);
    FragColor = blend(sky_color, sunset_color, smoothstep(0.0, 0.5, latitude)) * smoothstep(-0.3, 0.0, latitude);
    
    // run through stack backwards
    for (int i = reflections - 1; i >= 0; i--) {
        // get base color
        vec4 object_color = get_color(reflect_stack[i].model_index);
        vec3 surface_normal = reflect_stack[i].normal;

        // apply shading
        object_color = object_color * squash(smoothstep(-0.2, 1.0, dot(surface_normal, vec3(INV_SQRT2, INV_SQRT2, 0.0))), 0.2, 1.0);

        // blend with previous color (sky if no prev object)
        FragColor = blend(object_color, FragColor, get_reflectance(reflect_stack[i].model_index)); 
    }
}
