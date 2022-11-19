#ifndef PRIV_RENDERFRAME_H
#define PRIV_RENDERFRAME_H

#include <libavutil/frame.h>


void renderframeWithPixfmt(uint8_t* data[8], int* width, int* height, int linesize[8], AVFrame *frm, int avPixelFormat);

void renderframeAsRGBA(uint8_t* data[8], int* width, int* height, int linesize[8], AVFrame *frm);
void renderframeAsRGB(uint8_t* data[8], int* width, int* height, int linesize[8], AVFrame *frm);

#endif
