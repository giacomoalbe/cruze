#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 TexCoords;

out VS_OUPUT {
  flat uint gradient_type;
  float radius;
  vec4 first_color;
  vec4 calculated_position;
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

  calculated_position = vec4(Position.xy, 1.0, 1.0);

  if (is_textured == 1) {
    OUT.f_tex_pos = TexCoords;
    calculated_position = vec4(
        Position.x,
        bbox.r - Position.y,
        1.0,
        1.0
      );
  }

  gl_Position = projection * model * calculated_position;

  OUT.calculated_position = calculated_position;
  OUT.first_color = first_color;
  OUT.last_color = last_color;

  OUT.start_pos = start_pos;
  OUT.end_pos = end_pos;

  OUT.bbox = bbox;

  OUT.gradient_type = gradient_type;
  OUT.radius = radius;

  OUT.is_textured = is_textured;
}
