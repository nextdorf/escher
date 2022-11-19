#ifndef PRIV_DECODINGDECISION_H
#define PRIV_DECODINGDECISION_H


#include "DecodingDecision.h"


bool invokeDecodingDecider(const DecodingDecider *const decider, const DecodingDecisionIdx idx, const AVFormatContext *const fmt_ctx, const AVCodecContext *const codec_ctx,
  const AVStream *const stream, const AVPacket *const pkt, const AVFrame *const frm, 
  const struct SwsContext *const sws_ctx_if_scale, const AVFrame *const swsfrm, const int err);

void invokeDecodingActor(const DecodingActor *const actor, const DecodingActionIdx idx, AVFormatContext *const fmt_ctx, AVCodecContext *const codec_ctx,
  AVStream *const stream, AVPacket *const pkt, AVFrame *const frm, struct SwsContext *const sws_ctx_if_scale, 
  AVFrame *const swsfrm, const int err);


#endif


