#version 330 core

const uint LINEAR = uint(0);
const uint RADIAL = uint(1);

uniform sampler2D font_tex;

in VS_OUPUT {
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
} IN;

out vec4 Color;

void main() {
  /* BBox
   * TOP = r
   * RIGHT = g
   * BOTTOM = b
   * LEFT = a
   */
  float alpha = texture(font_tex, IN.f_tex_pos).r;

  float x = (IN.calculated_position.x - IN.bbox.a) / (IN.bbox.g - IN.bbox.a);
  float y = (IN.calculated_position.y - IN.bbox.b) / (IN.bbox.r - IN.bbox.b);

  float factor = 0.0;

  if (IN.gradient_type == LINEAR) {
    vec2 relative_position = vec2(x,y) - IN.start_pos;

    vec2 gradient_direction = IN.end_pos - IN.start_pos;

    factor = dot(relative_position, gradient_direction) /
             dot(gradient_direction, gradient_direction);
  } else {
      // start_pos is the center
      vec2 relative_position = IN.calculated_position.xy - IN.start_pos;

      factor = length(relative_position) / IN.radius;
  }

  Color = mix(IN.first_color, IN.last_color, clamp(factor, 0, 1));

  if (IN.is_textured == 1) {
    Color.a *= clamp(alpha, 0, 1);
  }

}
