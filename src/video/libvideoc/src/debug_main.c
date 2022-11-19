#include "renderframe.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>


int main(int argc, char** argv){
  const int skip_frames_default = 60*24;

  

  int width = -1, height = -1, linesize[8] = {-1};
  char *path;
  uint8_t *data[8] = {NULL};
  int skip_frames = -1;

  if(argc <= 1 || strcmp(argv[1], "--help") == 0){
    printf("Usage: %s path-to-video-file [skip-frames = %i]\n", argv[0], skip_frames_default);
    return 0;
  }
  if(argc >= 2)
    path = argv[1];
  if(argc >= 3)
    skip_frames = atoi(argv[2]);
  if(skip_frames < 0)
    skip_frames = skip_frames_default;

  printf("Input:\n  Path:\t %s\n  skip:\t %i frames\n", path, skip_frames);

  int res = renderfrom(path, data, &width, &height, linesize, skip_frames);

  printf("Result: %i\n  width:\t %i\n  height:\t %i\n", res, width, height);
  printf("  data set:\t [ %s", data[0] ? "True" : "False");
  for(int i=1; i<sizeof(data)/sizeof(*data); i++)
    printf(", %s", data[i] ? "True" : "False");
  printf(" ]\n");
  printf("  linesize:\t [ %i", linesize[0]);
  for(int i=1; i<sizeof(linesize)/sizeof(*linesize); i++)
    printf(", %i", linesize[i]);
  printf(" ]\n");


  FILE *f = fopen("raw.rgb", "wb");
  for(int i=0; i<sizeof(data)/sizeof(*data); i++){
    size_t size = linesize[i]*height;
    if(size)
      fwrite(data[i], 1, size, f);
  }
  fclose(f);

  for(int i=0; i<sizeof(data)/sizeof(*data); i++){
    uint8_t *d = data[i];
    if(d)
      free(d);
  }

  return res;
}


