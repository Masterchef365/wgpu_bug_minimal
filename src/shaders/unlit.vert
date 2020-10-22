#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

layout(binding = 0) uniform Camera {
    mat4 matrix;
} cam;

void main() {
    gl_Position = cam.matrix * vec4(inPosition, 1.0);
    fragColor = inColor;
}
