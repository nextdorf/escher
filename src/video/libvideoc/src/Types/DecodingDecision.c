#include <assert.h>
#include <string.h>
#include "DecodingDecision.h"


bool invokeDecodingDecider(const DecodingDecider *const decider, const DecodingDecisionIdx idx, const AVFormatContext *const fmt_ctx, const AVCodecContext *const codec_ctx,
    const AVStream *const stream, const AVPacket *const pkt, const AVFrame *const frm, 
    const struct SwsContext *const sws_ctx_if_scale, const AVFrame *const swsfrm, const int err){
  switch(decider->decisions[idx]){
    case DDecideTrue:
      return true;
    case DDecideFalse:
      return false;
    case DDecideFunctional:
      const DeciderFunc func = decider->params[idx];
      void *const state = decider->params[nDecodingDecisions];
      return func(fmt_ctx, codec_ctx, stream, pkt, frm, sws_ctx_if_scale, swsfrm, err, state);
    default:
      assert(false);
      break;
  }
}


void invokeDecodingActor(const DecodingActor *const actor, const DecodingActionIdx idx, AVFormatContext *const fmt_ctx, AVCodecContext *const codec_ctx,
    AVStream *const stream, AVPacket *const pkt, AVFrame *const frm, 
    struct SwsContext *const sws_ctx_if_scale, AVFrame *const swsfrm, const int err){
  switch(actor->actions[idx]){
    case DActNoop:
      break;
    case DActFunctional:
      const ActorFunc func = actor->params[idx];
      void *const state = actor->params[nDecodingActions];
      func(fmt_ctx, codec_ctx, stream, pkt, frm, sws_ctx_if_scale, swsfrm, err, state);
      break;
    default:
      assert(false);
      break;
  }
}

