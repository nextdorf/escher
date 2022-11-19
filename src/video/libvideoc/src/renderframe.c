#include "renderframe.h"
#include "priv_renderframe.h"
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
#include <libavutil/pixfmt.h>


int renderfrom(const char *path, 
  uint8_t* data[8], int* width, int* height, int linesize[8], 
  const int skip_frames) {
  AVFormatContext *fctx = NULL;
  AVCodecContext *cctx = NULL;
  int err;

  if ((err = avformat_open_input(&fctx, path, NULL, NULL)))
    return err;

  if ((err = avformat_find_stream_info(fctx, NULL)) < 0) {
    av_log(NULL, AV_LOG_ERROR, "Cannot find stream information\n");
    return err;
  }

  AVCodec *codec = NULL;
  int idx = 0;
  AVStream *stream = fctx->streams[idx];
  idx = av_find_best_stream(fctx, stream->codecpar->codec_type, idx, -1, (const AVCodec**)&codec, 0);
  stream = fctx->streams[idx];

  cctx = avcodec_alloc_context3(codec);
  avcodec_parameters_to_context(cctx, stream->codecpar);
  avcodec_open2(cctx, codec, NULL);

  int flagsTmp = fctx->flags | AVSEEK_FLAG_BACKWARD;
  int64_t tsTmp = (int64_t)(.1 * 60000.)+250000;
  // int64_t tsTmp = 251000;
  int errTmp;
  errTmp = av_seek_frame(fctx, idx, tsTmp, flagsTmp);


  AVPacket *pkt = av_packet_alloc();
  if(!pkt) return 0;
  AVFrame *frm = av_frame_alloc();
  if(!frm) {av_packet_free(&pkt); return 0;}

  // errTmp = av_seek_frame(fctx, idx, tsTmp, flagsTmp);

  uint8_t *ret = NULL;
  int skipFrames = skip_frames;
  while(!ret && (err = av_read_frame(fctx, pkt)) >= 0){
    if(pkt->stream_index != idx)
      continue;
    if((err = avcodec_send_packet(cctx, pkt)) < 0)
      break;

    av_frame_unref(frm);
    err = avcodec_receive_frame(cctx, frm);
    if(err >= 0){
      if(skipFrames > 0)
        --skipFrames;
      else{
        // renderframeAsRGBA(data, width, height, linesize, frm);
        renderframeWithPixfmt(data, width, height, linesize, frm, AV_PIX_FMT_RGB24);
        break;
      }
      //ret = frm->data;
    }
    else if(err == AVERROR(EAGAIN) || err == AVERROR_EOF)
      continue;
    else
      break;
  }
  av_frame_unref(frm);

  if(skipFrames > 0)
    err = -1;

  av_frame_free(&frm);
  av_packet_free(&pkt);
  avformat_close_input(&fctx);

  return err;
}

void renderframeWithPixfmt(uint8_t* data[8], int* width, int* height, int linesize[8], AVFrame *frm, int avPixelFormat){
  *width = frm->width;
  *height = frm->height;

  struct AVFrame * tmpFrm = av_frame_alloc();

  AV_PIX_FMT_YUV420P;
  struct SwsContext *swsCtx = sws_getContext(*width, *height, frm->format, *width, *height, avPixelFormat, SWS_BILINEAR, NULL, NULL, NULL);
  //see sws_frame_start(); sws_send_slice(0, src->height); sws_receive_slice(0, dst->height); sws_frame_end()
  //might be parrallizable
  sws_scale_frame(swsCtx, tmpFrm, frm);

  memcpy(linesize, tmpFrm->linesize, sizeof(tmpFrm->linesize));
  for(int i=0; i<sizeof(tmpFrm->data)/sizeof(*(tmpFrm->data)); ++i){
    if(tmpFrm->data[i]){
      const size_t size = *height * linesize[i];
      data[i] = malloc(size);
      memcpy(data[i], tmpFrm->data[i], size);
    }
    else
      data[i] = NULL;
  }

  av_frame_free(&tmpFrm);
  sws_freeContext(swsCtx);
}

void renderframeAsRGBA(uint8_t* data[8], int* width, int* height, int linesize[8], AVFrame *frm)
  { renderframeWithPixfmt(data, width, height, linesize, frm, AV_PIX_FMT_RGBA); }

void renderframeAsRGB(uint8_t* data[8], int* width, int* height, int linesize[8], AVFrame *frm)
  { renderframeWithPixfmt(data, width, height, linesize, frm, AV_PIX_FMT_RGB24); }


