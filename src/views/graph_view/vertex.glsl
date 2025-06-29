// SPDX-FileCopyrightText: 2025 Menno van der Graaf <mennovandergraaf@hotmail.com>
// SPDX-License-Identifier: MIT

uniform mat3 view_transform;
attribute vec2 coordinates;
attribute float point_size;
attribute vec3 color;

varying vec4 f_color;

void main(void) {
    f_color = vec4(color.r, color.g, color.b, 1.0);
    vec3 transformed_vertex = view_transform * vec3(coordinates, 1.0);
    gl_Position = vec4(transformed_vertex, 1.0);
    gl_PointSize = point_size;
}
