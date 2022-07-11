#version 330 core

#define PI 3.141592653589793
#define SQRT2 1.4142135623730951
#define INV_SQRT2 0.7071067811865475
#define MB_STEPS 200

in vec2 ScreenPos;
out vec4 FragColor;

uniform float uTime;                // time since start, sec
uniform float uRatio;               // width:height ratio
uniform ivec2 uScreenResolution;    // screen-resolution in pixels, int

vec2 c_pow2(vec2 z)
{
   return vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y); 
}

vec2 mb_np1(vec2 z_n, vec2 c)
{
    return c_pow2(z_n) + c;
}

float erf_peak(float x, float a, float b)
{
    return exp2(-pow(a * (x - b), 2.0));
}

struct mb_res 
{
    int steps;
    bool is_in;
};

mb_res mb_nr_steps(vec2 c)
{
    vec2 z = vec2(0.0,0.0);
    
    for (int steps = 0; steps < MB_STEPS; steps++)
    {
        z = mb_np1(z, c);    
        if (length(z) > 2.0) {
            return mb_res(steps, false);
        }
    }
    return mb_res(0, true);
}

void main()
{
    vec2 pos = vec2(ScreenPos.x * uRatio, ScreenPos.y) * 2.0;
    mb_res mb = mb_nr_steps(pos);
    float color = 0;
    if (!mb.is_in) {
        color = pow(mb.steps / float(MB_STEPS) - 1.0, 3.0) ;
    }
    FragColor = vec4(
        erf_peak(color, 10.0, 0.2), 
        erf_peak(color,  3.9, 0.5), 
        erf_peak(color,  9.7, 0.8), 
        1.0);
}
