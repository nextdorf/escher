#ifndef VIDEOC_H
#define VIDEOC_H

#include <stdint.h>
#include <stdlib.h>


uint8_t *genSomeData(const size_t len);

void freeData(uint8_t *data);

#endif

