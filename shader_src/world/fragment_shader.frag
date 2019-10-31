#version 400

uniform vec3 ambient_light_color;
uniform vec3 directional_light_dir;
uniform vec3 directional_light_color;
uniform sampler2D tex;
uniform vec3 color;

vec3 normalized_directional_light_dir = normalize(directional_light_dir);

in vec3 gf_normal;
in vec2 gf_texture_position;

out vec4 out_color;

void main() {
    vec3 light = max(0.0, -dot(gf_normal, normalized_directional_light_dir)) * directional_light_color;
    light += ambient_light_color;
    out_color = vec4(color * light, 1.0) * texture(tex, gf_texture_position);
}
