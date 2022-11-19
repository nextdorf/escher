#include "videoc.h"
#include "priv_videoc.h"
#include <stdio.h>

uint8_t *genSomeData(const size_t len){
  uint8_t *data = (uint8_t *)malloc(len);
  for(int i=0; i<len; ++i)
    data[i] = i % 2;
  return data;
}

void freeData(uint8_t *data){
  free(data);
}

void unknown_fn(){
  printf("Don't reverse engineer me, I'm shy");
}
