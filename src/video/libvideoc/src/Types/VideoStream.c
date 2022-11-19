#include "VideoStream.h"
#include "priv_DecodingDecision.h"
#include <assert.h>


VideoStreamResult vs_open_format_context_from_path(char *path, AVFormatContext **fmt_ctx, int *err){
  if((*err = avformat_open_input(fmt_ctx, path, NULL, NULL)) < 0)
    return vs_ffmpeg_errorcode;
  if((*err = avformat_find_stream_info(*fmt_ctx, NULL)) < 0)
    return vs_io;
  return vs_success;
}

VideoStreamResult vs_open_codec_context(AVFormatContext *fmt_ctx, int stream_idx, uint32_t nThreads, int resolution, AVCodecContext **codec_ctx, int *err){
  /*
  Let Rust call av_find_default_stream_index, av_find_best_stream in any way and specify the behaivour with enums

  Programs are essentially a "bundle of streams" (see https://en.wikipedia.org/wiki/MPEG_transport_stream#Programs) and by
  specifying related stream in av_find_best_stream FFMPEG trys to find a stream in the same program (https://ffmpeg.org/doxygen/5.1/avformat_8c_source.html#l00347)
  */
  if(stream_idx < 0 || stream_idx >= fmt_ctx->nb_streams)
    return vs_index_out_of_bounds;
  AVStream *stream = fmt_ctx->streams[stream_idx];
  AVCodec *codec = NULL;
  // Internal ff_find_decoder() is not part of public ABI
  *err = av_find_best_stream(fmt_ctx, stream->codecpar->codec_type, stream_idx, -1, (const AVCodec**)&codec, 0);
  if(*err != stream_idx)
    switch(*err){
      case AVERROR_DECODER_NOT_FOUND: //AVERROR_DECODER_NOT_FOUND if streams were found but no decoder
        return vs_decoder_not_found;
      case AVERROR_STREAM_NOT_FOUND: //AVERROR_STREAM_NOT_FOUND if no stream with the requested type could be found
      default: //wrong stream found(?)
        return vs_stream_not_found;
    }

  *codec_ctx = avcodec_alloc_context3(codec);


  if(resolution>=0)
    (*codec_ctx)->lowres = resolution < codec->max_lowres ? resolution : codec->max_lowres;
  else
    (*codec_ctx)->lowres = -resolution > codec->max_lowres ? 0 : codec->max_lowres+1+resolution;
  (*codec_ctx)->thread_count = nThreads;
  // (*codec_ctx)->thread_type = FF_THREAD_SLICE;


  if((*err = avcodec_parameters_to_context(*codec_ctx, stream->codecpar)) < 0)
    return vs_ffmpeg_errorcode;
  if((*err = avcodec_open2(*codec_ctx, codec, NULL)) < 0)
    return vs_ffmpeg_errorcode;

  // If the following asserts hold there is need for exporting the codec
  // assert((*codec_ctx)->codec->id == codec->id);
  assert(!strcmp((*codec_ctx)->codec->name, codec->name));
  assert(av_codec_is_encoder((*codec_ctx)->codec) == av_codec_is_encoder(codec));
  assert(av_codec_is_decoder((*codec_ctx)->codec) == av_codec_is_decoder(codec));

  return vs_success;
}

VideoStreamResult vs_create_sws_context(AVCodecContext *codec_ctx, struct SwsContext **sws_ctx,
  int new_width, int new_height, enum AVPixelFormat new_pix_fmt, int flags, const double *param, int *err){
  const int width = codec_ctx->width;
  const int height = codec_ctx->height;
  const int newwidth = new_width >= 0 ? new_width : width;
  const int newheight = new_height >= 0 ? new_height : height;
  *sws_ctx = sws_getContext(
    width, height, codec_ctx->pix_fmt, 
    newwidth, newheight, new_pix_fmt,
    flags, NULL, NULL, param );
  return *sws_ctx ? vs_success : vs_ffmpeg_errorcode;
}

VideoStreamResult vs_create_pkt_frm(AVPacket **pkt, AVFrame **frm, AVFrame **swsfrm){
  if(!(pkt && frm && swsfrm))
    return vs_null_reference;
  
  *pkt = av_packet_alloc();
  *frm = av_frame_alloc();
  *swsfrm = av_frame_alloc();
  return vs_success;
}


void vs_free(VideoStream *vstream) {
  av_frame_unref(vstream->frm);
  av_frame_unref(vstream->swsfrm);
  av_frame_free(&(vstream->frm));
  av_frame_free(&(vstream->swsfrm));
  av_packet_unref(vstream->pkt);
  av_packet_free(&vstream->pkt);
  sws_freeContext(vstream->sws_ctx);
  avcodec_close(vstream->codec_ctx);
  avformat_close_input(&(vstream->fmt_ctx));
}



bool vs_seek_ddecide(DeciderFuncParams) {
  const int64_t *timestamp = state;
  return *timestamp >= frm->pts + frm->pkt_duration;
}
void vs_seek_dact(ActorFuncParams) {
  const int64_t timestamp = *((int64_t *)state);
  const enum AVDiscard skip_value = *((enum AVDiscard*)(state + sizeof(int64_t)));
  if(   timestamp < pkt->pts + 2*pkt->duration //pkt?
    &&  skip_value != codec_ctx->skip_frame)
    codec_ctx->skip_frame = skip_value;
}
VideoStreamResult vs_seek(AVFormatContext *fmt_ctx, AVStream *stream, int64_t timestamp, int flags, AVCodecContext *codec_ctx_if_decode_frames, AVPacket *pkt, AVFrame *frm, int* err){
  *err = 0;

  if(flags >= 0){
    // Fast mode
    if(codec_ctx_if_decode_frames)
      flags |= AVSEEK_FLAG_BACKWARD;// | AVSEEK_FLAG_ANY;
    flags |= fmt_ctx->flags;
    *err = av_seek_frame(fmt_ctx, stream->index, timestamp, flags);
    if(*err < 0) return vs_ffmpeg_errorcode;
  }

  if(codec_ctx_if_decode_frames) {
    // Precise mode
    VideoStreamResult res = vs_success;

    // while(frm->pts == AV_NOPTS_VALUE || timestamp >= frm->pts + frm->pkt_duration) {
    //   res = vs_decode_next_frame(fmt_ctx, codec_ctx_if_decode_frames, stream, pkt, frm, NULL, NULL, err);
    //   if(res != vs_success && res != vs_eof)
    //     return res;
    // }
    DecodingDecider decider = new_DecodingDecider();
    decider.decisions[DDecideDecodeFrame] = DDecideFunctional;
    decider.params[DDecideDecodeFrame] = vs_seek_ddecide;
    decider.decisions[DDecideDecodePacket] = DDecideFunctional;
    decider.params[DDecideDecodePacket] = vs_seek_ddecide;
    decider.params[nDecodingDecisions] = &timestamp;
    DecodingActor actor = new_DecodingActor();
    actor.actions[DActDecodePktToCodecCtxPrepare] = DActFunctional;
    actor.params[DActDecodePktToCodecCtxPrepare] = vs_seek_dact;

    const enum AVDiscard skip_value = codec_ctx_if_decode_frames->skip_frame;
    uint8_t paramBuffer[sizeof(timestamp) + sizeof(skip_value)];
    memcpy(paramBuffer, &timestamp, sizeof(timestamp));
    memcpy(paramBuffer + sizeof(timestamp), &skip_value, sizeof(skip_value));
    actor.params[nDecodingActions] = paramBuffer;
    codec_ctx_if_decode_frames->skip_frame = AVDISCARD_DEFAULT;

    res = vs_decode(fmt_ctx, codec_ctx_if_decode_frames, stream, pkt, frm, NULL, NULL, &decider, &actor, err);

    codec_ctx_if_decode_frames->skip_frame = skip_value;
    if(res != vs_success && res != vs_eof)
      return res;

    if(timestamp > frm->pts + frm->pkt_duration)
      return vs_timestamp_out_of_bounds;
    if(frm->pts == AV_NOPTS_VALUE)
      return vs_ffmpeg_errorcode;
  }
  return vs_success;
}

VideoStreamResult vs_seek_at(AVFormatContext *fmt_ctx, AVStream *stream, double seconds, int flags, AVCodecContext *codec_ctx_if_decode_frames, AVPacket *pkt, AVFrame *frm, int* err){
  const int64_t timestamp = seconds * stream->time_base.den/stream->time_base.num;
  return vs_seek(fmt_ctx, stream, timestamp, flags, codec_ctx_if_decode_frames, pkt, frm, err);
}


#define doDecide(idx) \
  invokeDecodingDecider(decider, idx, fmt_ctx, codec_ctx, stream, pkt, frm, sws_ctx, swsfrm, *err)

#define doAct(idx) \
  invokeDecodingActor(actor, idx, fmt_ctx, codec_ctx, stream, pkt, frm, sws_ctx, swsfrm, *err)


VideoStreamResult vs_decode(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm,
    struct SwsContext *sws_ctx, AVFrame *swsfrm, const DecodingDecider *const decider, const DecodingActor *const actor, int *err){
  doAct(DActDecodeInit);
  bool mustDecodePacket = false;
  // if(   nFrames > 0 //DDecideDecodeFrameIdx
  if(   doDecide(DDecideDecodeFrame)
    ||  pkt->dts != frm->pkt_dts
    // ||  pkt->pts != frm->pts
    ||  frm->pkt_dts == AV_NOPTS_VALUE ) {
    do {
      if(mustDecodePacket || doDecide(DDecideDecodePacket)){
        doAct(DActDecodeFmtCtxToPktPrepare);
        av_packet_unref(pkt);
        *err = av_read_frame(fmt_ctx, pkt);
        switch(*err){
          case 0:
            doAct(DActDecodeFmtCtxToPktSuccess);
            break;
          // //Should never happen
          // case AVERROR(EAGAIN):
          //   continue;
          case AVERROR_EOF:
            return vs_eof;
          default:
            return vs_ffmpeg_errorcode;
        }

        // //Used to continue if this is false (copied from a tutorial), but the more I think about it the less sense it makes
        // assert(stream->pkt->stream_index == stream->stream->index);
        //Yeah the index does indeed change, not sure what it means tho. Sound catching up?
        if(pkt->stream_index != stream->index)
          continue;

        doAct(DActDecodePktToCodecCtxPrepare); //Unnecessary? Since doAct(DActDecodeFmtCtxToPktSuccess) was always already called
        *err = avcodec_send_packet(codec_ctx, pkt);
        switch(*err){
          case AVERROR(EAGAIN): //input is not accepted in the current state - user must read output with avcodec_receive_frame() (once all output is read, the packet should be resent, and the call will not fail with EAGAIN).
            // noop in switch, instead of continueing the loop because this errorcode means here
            // that the frame was only partially read, i.e. the packet was already sent to codec_ctx
            // but wasn't received yet by the frame
            // TODO: Actually check if the above comment is true and we shouldn't do: avcodec_receive_frame -> avcodec_send_packet -> break switch
            doAct(DActDecodePktToCodecCtxEAGAIN);
            break;
          case 0:
            doAct(DActDecodePktToCodecCtxSuccess);
            break;
          case AVERROR(AVERROR_EOF): //the decoder has been flushed, and no new packets can be sent to it (also returned if more than 1 flush packet is sent)
            return vs_eof;
          case AVERROR(EINVAL): //codec not opened, it is an encoder, or requires flush
            if(av_codec_is_encoder(codec_ctx->codec))
              return vs_encoder_trys_to_decode;
            else
              return vs_ffmpeg_errorcode;
          default:
            return vs_ffmpeg_errorcode;
        }
      }

      mustDecodePacket = false;
      doAct(DActDecodeCodecCtxToFrmPrepare);
      av_frame_unref(frm);
      *err = avcodec_receive_frame(codec_ctx, frm);
      switch (*err) {
        case 0:
          doAct(DActDecodeCodecCtxToFrmSuccess);
          break;
        case AVERROR(EAGAIN): // output is not available in this state - user must try to send new input
          //What if there is no new input??? Still mustDecodePacket = true?
          mustDecodePacket = true;
          doAct(DActDecodeCodecCtxToFrmEAGAIN);
          break;
        case AVERROR_EOF: // the decoder has been fully flushed, and there will be no more output frames
          return vs_eof;
        default:
          // AVERROR(EINVAL): codec not opened, or it is an encoder
          // AVERROR_INPUT_CHANGED: current decoded frame has changed parameters with respect to first decoded frame. Applicable when flag AV_CODEC_FLAG_DROPCHANGED is set.
          // other negative values: legitimate decoding errors
          return vs_ffmpeg_errorcode;
      }
    } while(mustDecodePacket || doDecide(DDecideDecodeFrame));

    doAct(DActDecodeFrameDone);
  }
  // if(sws_ctx_if_scale && swsfrm->pts != frm->pts){
  if(   doDecide(DDecideDecodeSws)
    && (doDecide(DDecideDecodeSwsIgnorePts) || swsfrm->pts != frm->pts)
    ){
    doAct(DActDecodeSwsFrmPrepare);
    av_frame_unref(swsfrm);
    *err = sws_scale_frame(sws_ctx, swsfrm, frm);
    if(*err < 0)
      return vs_ffmpeg_errorcode;
    swsfrm->best_effort_timestamp = frm->best_effort_timestamp;
    swsfrm->pts = frm->pts;
    swsfrm->pkt_dts = frm->pkt_dts;
    swsfrm->pkt_duration = frm->pkt_duration;
    doAct(DActDecodeSwsFrmSuccess);
  }
  return vs_success;
}

VideoStreamResult vs_decode_current_frame(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm, struct SwsContext *sws_ctx, AVFrame *swsfrm, int *err){
  return vs_decode_frames(fmt_ctx, codec_ctx, stream, pkt, frm, sws_ctx, swsfrm, 0, err);
}

VideoStreamResult vs_decode_next_frame(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm, struct SwsContext *sws_ctx, AVFrame *swsfrm, int *err){
  return vs_decode_frames(fmt_ctx, codec_ctx, stream, pkt, frm, sws_ctx, swsfrm, 1, err);
}



bool vs_decode_frames_ddecide(DeciderFuncParams) {
  const uint64_t *nFrames = state;
  return *nFrames > 0;
}
void vs_decode_frames_dact_frm(ActorFuncParams) {
  uint64_t *nFrames = state;
  if(*nFrames)
    --(*nFrames);
}
VideoStreamResult vs_decode_frames(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm, struct SwsContext *sws_ctx_if_scale, AVFrame *swsfrm, uint64_t nFrames, int *err){
  DecodingDecider decider = new_DecodingDecider();
  decider.decisions[DDecideDecodeFrame] = DDecideFunctional;
  decider.params[DDecideDecodeFrame] = vs_decode_frames_ddecide;
  decider.decisions[DDecideDecodePacket] = DDecideFunctional;
  decider.params[DDecideDecodePacket] = vs_decode_frames_ddecide;
  decider.decisions[DDecideDecodeSws] = sws_ctx_if_scale ? DDecideTrue : DDecideFalse;
  decider.params[nDecodingDecisions] = &nFrames;
  DecodingActor actor = new_DecodingActor();
  actor.actions[DActDecodeCodecCtxToFrmSuccess] = DActFunctional;
  actor.params[DActDecodeCodecCtxToFrmSuccess] = vs_decode_frames_dact_frm;
  actor.params[nDecodingActions] = &nFrames;
  return vs_decode(fmt_ctx, codec_ctx, stream, pkt, frm, sws_ctx_if_scale, swsfrm, &decider, &actor, err);
}



