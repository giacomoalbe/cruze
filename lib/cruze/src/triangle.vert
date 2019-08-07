#version 330 core
const mat4 INVERT_Y_AXIS = mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, -1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0)
);

layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 left_top;
layout (location = 2) in vec2 right_bottom;
layout (location = 3) in vec2 tex_left_top;
layout (location = 4) in vec2 tex_right_bottom;

out VS_OUPUT {
  flat uint gradient_type;
  float radius;
  vec4 first_color;
  vec4 last_color;
  vec2 start_pos;
  vec2 end_pos;
  vec4 bbox;
  vec2 f_tex_pos;
  flat int is_textured;
} OUT;


uniform mat4 projection;
uniform mat4 model;

uniform vec4 first_color;
uniform vec4 last_color;
uniform vec2 start_pos;
uniform vec2 end_pos;
uniform vec4 bbox;
uniform uint gradient_type;
uniform float radius;
uniform int is_textured;

void main() {
  vec4 calculated_position;

  if (is_textured == 1) {
    vec2 pos = vec2(0.0);

    float left = left_top.x;
    float right = right_bottom.x;
    float top = left_top.y;
    float bottom = right_bottom.y;

    switch (gl_VertexID) {
        case 0:
            pos = vec2(left, top);
            OUT.f_tex_pos = vec2(tex_left_top.x, tex_right_bottom.y);
            break;
        case 1:
            pos = vec2(right, top);
            OUT.f_tex_pos = tex_right_bottom;
            break;
        case 2:
            pos = vec2(left, bottom);
            OUT.f_tex_pos = tex_left_top;
            break;
        case 3:
            pos = vec2(right, bottom);
            OUT.f_tex_pos = vec2(tex_right_bottom.x, tex_left_top.y);
            break;
    }

    //f_color = color;
    calculated_position = vec4(pos, left_top.z, 1.0);
  } else {
    calculated_position = vec4(Position.xyz, 1.0);
  }

  gl_Position = projection * model * calculated_position;

  OUT.first_color = first_color;
  OUT.last_color = last_color;

  OUT.start_pos = start_pos;
  OUT.end_pos = end_pos;

  OUT.bbox = bbox;

  OUT.gradient_type = gradient_type;
  OUT.radius = radius;

  OUT.is_textured = is_textured;
}
