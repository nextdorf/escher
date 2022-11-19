#ifndef RENDERFRAME_H
#define RENDERFRAME_H

#include <stdint.h>

int renderfrom(const char *path, 
  uint8_t* data[8], int* width, int* height, int linspace[8], 
  const int skip_frames);

#endif
