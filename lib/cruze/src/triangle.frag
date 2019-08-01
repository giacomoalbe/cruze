#version 330 core

in VS_OUPUT {
  vec3 first_color;
  vec3 last_color;
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

  vec4 first_color = vec4(IN.first_color, 1.0);
  vec4 last_color = vec4(IN.last_color, 1.0);

  vec2 start_pos = IN.start_pos;
  vec2 end_pos = IN.end_pos;

  vec2 relative_position = vec2(x,y) - start_pos;

  vec2 gradient_direction = end_pos - start_pos;

  float factor = dot(relative_position, gradient_direction) /
                 dot(gradient_direction, gradient_direction);

  Color = mix(first_color, last_color, factor);
}
