// stolen from https://github.com/bevyengine/bevy/blob/latest/assets/shaders/custom_material.frag
#version 450
layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform vec4 CustomMaterial_color;

layout(set = 2, binding = 1) uniform texture2D CustomMaterial_texture;
layout(set = 2, binding = 2) uniform sampler CustomMaterial_sampler;

vec4 v4(vec3 v3) {
    return vec4(v3, 1.);
}

float xy(vec2 v2) {
    return abs(v2.x - v2.y);
//    return v2.x * v2.y;
}

void main() {
    // o_Target = PbrFuncs::tone_mapping(Color * texture(sampler2D(CustomMaterial_texture,CustomMaterial_sampler), v_Uv));
    vec2 muv = mod(v_Uv, vec2(1.));
    vec2 tuv = muv * 0.5;

    // road stripes
    float stripe_thickness = 0.008;
    float stripe = 1. - step(stripe_thickness * 2., abs(abs(muv.y - 0.5) - 0.35));
    vec3 yellow = vec3(1., 1., 1.);
    vec3 stripe_col = mix(vec3(1.), yellow, stripe);
//    o_Target = CustomMaterial_color * muv.xyxy;
    vec4 tcolor = texture(sampler2D(CustomMaterial_texture, CustomMaterial_sampler), tuv);
    vec3 color = tcolor.xyz * stripe_col + yellow * stripe * 0.8;


    // start line
    float checker_size = 0.06;
    float schecker = 1. - step(checker_size * 2., v_Uv.x); // step(v_Uv.x, 0.3333);
    vec3 checker = vec3(xy(step(0.5 * checker_size, mod(muv, checker_size))));
    //color = checker;
    color += schecker * checker;
    o_Target = vec4(CustomMaterial_color.xyz * color.xyz, 1.);
}
