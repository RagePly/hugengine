#version 460 core

#define PI 3.141592653589793
#define SQRT2 1.4142135623730951
#define INV_SQRT2 0.7071067811865475
#define MB_STEPS 200


in vec2 ScreenPos;
out vec4 FragColor;

uniform float uTime;                // time since start, sec
uniform float uRatio;               // width:height ratio
uniform ivec2 uScreenResolution;    // screen-resolution in pixels, int

// NOTE: this binding is statically typed in the shaderdevprogram.
layout(std430, binding = 4) buffer ModelIndex 
{
    int uModelIndex[];
};

layout(std430, binding = 5) buffer ModelProperties 
{
    float uModelProps[];
};

struct Properties {
    // Transform
    vec3 position;
    float scale;
    vec3 rotation;

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

vec3 get_collor3(int i) {
    return vec3(uModelProps[i+7], uModelProps[i+8], uModelProps[i+9]);
}

vec4 get_color(int i) {
    return vec4(uModelProps[i+7], uModelProps[i+8], uModelProps[i+9], 1.0);
}

// populate the Properties-struct `prop` for the object `i`
#define fetch_props(prop, i)                \
    prop.position = get_position3(i);       \
    prop.scale = get_scale(i);              \
    prop.rotation = vec3(                   \
            uModelProps[i+4],               \
            uModelProps[i+5],               \
            uModelProps[i+6]                \
            );                              \
    prop.color = vec3(                      \
            uModelProps[i+7],               \
            uModelProps[i+8],               \
            uModelProps[i+9]                \
            );                              \
    prop.reflectance = uModelProps[i+10];   

#define SPHERE_ID 0
#define BOX_ID 1
#define PLANE_ID 2

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
    normal = vec3(0.0, 1.0, 0.0); // TODO: change this to match rotation-transform
    return intersection_plane(
            ray_o, 
            ray_d, 
            get_position3(i),
            normal,
            t_max,
            t_intersect);
}

// creates a ray pointing to the -z direction
void create_ray(in float fov_deg, out vec3 ray_o, out vec3 ray_d) {
    float half_fov_rad = deg2rad(fov_deg / 2.0);
    float z = uRatio / tan(half_fov_rad);

    ray_o = vec3(uRatio * ScreenPos.x, ScreenPos.y, 0.0);
    ray_d = normalize(ray_o - vec3(0.0,0.0,z));
}

void main()
{
    FragColor = vec4(0.0,0.0,0.0,1.0);


    vec4 c = vec4(0.0,0.0,0.0,0.0);
    float t_max = 1000.0;
    float t_smallest = t_max;
    int i_smallest = 0;

    vec3 ray_o, ray_d, reflection_normal;
    create_ray(90.0, ray_o, ray_d);
    vec2 p = vec2(uRatio * ScreenPos.x, ScreenPos.y);
    bool intersection_found = false;
    for (int i = 0; i < uModelIndex.length() / 2; i++) 
    {
        int model_type = uModelIndex[2 * i];
        int prop_index = uModelIndex[2 * i + 1];
        float t_intersect;

        switch (model_type) {
            case SPHERE_ID: // sphere
                if (draw_sphere(prop_index, 
                            ray_o, 
                            ray_d, 
                            1000.0,
                            t_intersect,
                            reflection_normal)
                   ) 
                {
                    i_smallest = t_intersect < t_smallest ? i : i_smallest;
                    t_smallest = t_intersect < t_smallest ? t_intersect : t_smallest;
                    intersection_found = true;
                }
                break;
            case PLANE_ID: // box
                if (draw_plane(prop_index, 
                            ray_o, 
                            ray_d, 
                            1000.0,
                            t_intersect,
                            reflection_normal)
                   ) 
                {
                    i_smallest = t_intersect < t_smallest ? i : i_smallest;
                    t_smallest = t_intersect < t_smallest ? t_intersect : t_smallest;
                    intersection_found = true;
                }
                break;
            default:
                break;
        }
    }

    // draw sky
    vec4 sky_color = rgb(85, 170, 224);
    vec4 sunset_color = rgb(246, 109, 73);
    FragColor = blend(sky_color, sunset_color, smoothstep(0.0, 0.5, p.y)) * smoothstep(-0.3, 0.0, p.y);

    if (intersection_found) {
        int draw_model = uModelIndex[2 * i_smallest];
        int draw_index = uModelIndex[2 * i_smallest + 1];

        switch (draw_model) {
            case SPHERE_ID:
                FragColor = get_color(draw_index);
                break;
            case PLANE_ID:
                FragColor = get_color(draw_index);
                break;
            default:
                break;
        }
    }
}
