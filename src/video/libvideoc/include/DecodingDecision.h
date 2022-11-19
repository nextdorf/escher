#ifndef UTILS_H
#define UTILS_H

#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
#include <stdbool.h>


enum DecodingDecision {
  DDecideFalse=0,
  DDecideTrue=1,
  DDecideFunctional,
};
typedef enum DecodingDecision DecodingDecision;

enum DecodingDecisionIdx{
  DDecideDecodeFrame,
  DDecideDecodePacket,
  DDecideDecodeSws,
  DDecideDecodeSwsIgnorePts,
  nDecodingDecisions, // Represents the number of enum entrys as long as it is the last one
};
typedef enum DecodingDecisionIdx DecodingDecisionIdx;

struct DecodingDecider {
  DecodingDecision decisions[nDecodingDecisions];
  void *params[nDecodingDecisions+1];
};
typedef struct DecodingDecider DecodingDecider;

#define new_DecodingDecider() \
    (DecodingDecider){ {DDecideFalse}, {NULL} }



enum DecodingAction {
  DActNoop,
  DActFunctional,
};
typedef enum DecodingAction DecodingAction;

enum DecodingActionIdx{
  DActDecodeInit,
  DActDecodeFmtCtxToPktPrepare,
  DActDecodeFmtCtxToPktSuccess,
  DActDecodePktToCodecCtxPrepare,
  DActDecodePktToCodecCtxSuccess,
  DActDecodePktToCodecCtxEAGAIN,
  DActDecodeCodecCtxToFrmPrepare,
  DActDecodeCodecCtxToFrmSuccess,
  DActDecodeCodecCtxToFrmEAGAIN,
  DActDecodeFrameDone,
  DActDecodeSwsFrmPrepare,
  DActDecodeSwsFrmSuccess,
  nDecodingActions, // Represents the number of enum entrys as long as it is the last one
};
typedef enum DecodingActionIdx DecodingActionIdx;

struct DecodingActor {
  DecodingAction actions[nDecodingActions];
  void *params[nDecodingActions+1];
};
typedef struct DecodingActor DecodingActor;

#define new_DecodingActor() \
    (DecodingActor){ {DActNoop}, {NULL} }



#define DeciderFuncParams const AVFormatContext *const fmt_ctx, const AVCodecContext *const codec_ctx,\
  const AVStream *const stream, const AVPacket *const pkt, const AVFrame *const frm, \
  const struct SwsContext *const sws_ctx, const AVFrame *const swsfrm, const int err, \
  void *const state

typedef bool (*DeciderFunc)(DeciderFuncParams);


#define ActorFuncParams AVFormatContext *const fmt_ctx, AVCodecContext *const codec_ctx, \
  AVStream *const stream, AVPacket *const pkt, AVFrame *const frm, struct SwsContext *const sws_ctx, \
  AVFrame *const swsfrm, const int err, void *const state

typedef void (*ActorFunc)(ActorFuncParams);



#endif

