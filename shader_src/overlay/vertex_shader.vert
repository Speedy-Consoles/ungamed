#version 400

uniform mat3 object_to_screen_matrix;

in vec2 position;
in vec2 texture_position;

out vec4 vg_screen_position;
out vec2 vg_texture_position;

void main() {
    vg_screen_position = vec4((object_to_screen_matrix * vec3(position, 1.0)).xy, 0.0, 1.0);
    vg_texture_position = texture_position;
}
