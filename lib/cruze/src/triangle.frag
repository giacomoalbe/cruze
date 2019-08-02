#version 330 core

const uint LINEAR = uint(0);
const uint RADIAL = uint(1);

in VS_OUPUT {
  flat uint gradient_type;
  float radius;
  vec4 first_color;
  vec4 last_color;
  vec2 start_pos;
  vec2 end_pos;
  vec4 bbox;
} IN;

out vec4 Color;

void main() {
  /* BBox
   * TOP = r
   * RIGHT = g
   * BOTTOM = b
   * LEFT = a
   */

  float x = (gl_FragCoord.x - IN.bbox.a) / (IN.bbox.g - IN.bbox.a);
  float y = (gl_FragCoord.y - IN.bbox.b) / (IN.bbox.r - IN.bbox.b);

  float factor = 0.0;

  if (IN.gradient_type == LINEAR) {
    vec2 relative_position = vec2(x,y) - IN.start_pos;

    vec2 gradient_direction = IN.end_pos - IN.start_pos;

    factor = dot(relative_position, gradient_direction) /
             dot(gradient_direction, gradient_direction);
  } else {
      // start_pos is the center
      vec2 relative_position = gl_FragCoord.xy - IN.start_pos;

      factor = length(relative_position) / IN.radius;
  }

  Color = mix(IN.first_color, IN.last_color, factor);
}
