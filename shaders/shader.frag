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
vec4 fetch_position(int i)
{
    return vec4(uModelProps[i],
                uModelProps[i+1],
                uModelProps[i+2],
                1.0
           );
}

// fetch vec3 of the position for the object `i`
vec3 fetch_position3(int i)
{
    return vec3(uModelProps[i],
                uModelProps[i+1],
                uModelProps[i+2]
           );
}

// populate the Properties-struct `prop` for the object `i`
#define fetch_props(prop, i)                \
    prop.position = fetch_position3(i);     \
    prop.scale = uModelProps[i+3];          \
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


vec4 draw_sphere(int i, vec2 pos)
{
    // fetch data
    Properties props;
    fetch_props(props, i);
    if (length(pos - props.position.xy) < props.scale) {
        return vec4(props.color, 1.0);
    } else {
        return vec4(0.0,0.0,0.0,0.0);
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

void main()
{
    vec2 p = vec2(uRatio * ScreenPos.x, ScreenPos.y);
    vec4 c = vec4(0.0,0.0,0.0,0.0);
    for (int i = 0; i < uModelIndex.length() / 2; i++) 
    {
        int model_type = uModelIndex[2 * i];
        int prop_index = uModelIndex[2 * i + 1];

        switch (model_type) {
            case 0: // sphere
                c += draw_sphere(prop_index, p);
                break;
            case 1: // box
                c += draw_box(prop_index, p);
                break;
            default:
                break;
        }
    }
    FragColor = c;
}
