//#version 430 core

//out vec4 color;

//void main()
//{
//    color = vec4(1.0f, 1.0f, 1.0f, 1.0f);
//}


// ##################################################################
// Task 2 d)ii) Change the triangle color

//#version 430 core

//out vec4 color;

//void main()
//{
//    color = vec4(1.0, 0.0, 0.0, 1.0);
//}

// ##################################################################
// Task 3 d) Draw a shape and have its colour change slowly over time

#version 430 core

uniform float uTime;   // seconds since start
out vec4 color;

void main()
{
    float r = 0.5 + 0.5 * sin(uTime * 0.8);
    float g = 0.5 + 0.5 * sin(uTime * 0.8 + 2.094); // +120°
    float b = 0.5 + 0.5 * sin(uTime * 0.8 + 4.188); // +240°
    color = vec4(r, g, b, 1.0);
}
// ##################################################################
