#version 430 core

layout (location = 0) in vec3 position;

void main()
{
    gl_Position = vec4(position, 1.0);
}

// ##################################################################
// Task 2 d)i) Flip whole scene horizontally

//#version 430 core

//in vec3 position;

//void main()
//{
//    vec4 p = vec4(position, 1.0);
//    p.xy *= -1.0;
//    gl_Position = p;
//}
// ##################################################################