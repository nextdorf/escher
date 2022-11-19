#ifndef VIDEOSTREAM_H
#define VIDEOSTREAM_H


#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
#include <libavutil/pixfmt.h>
#include <stdbool.h>
#include "DecodingDecision.h"


/** <div rustbindgen private>
 * <div rustbindgen nocopy>
*/
struct VideoStream
{
  AVFormatContext *fmt_ctx;
  AVCodecContext *codec_ctx;
  AVStream *stream;

  AVPacket *pkt;
  AVFrame *frm;
  struct SwsContext *sws_ctx;
  AVFrame *swsfrm;
};

enum VideoStreamResult{
  vs_ffmpeg_errorcode = -1,
  vs_success = 0,
  // vs_fmt_ctx_is_none,
  // vs_codec_is_none,
  // vs_codec_ctx_is_none,
  // vs_stream_is_none,

  vs_timestamp_out_of_bounds,
  vs_eof,
  vs_io,
  vs_encoder_trys_to_decode,
  vs_decoder_trys_to_encode,
  vs_index_out_of_bounds,
  vs_stream_not_found,
  vs_decoder_not_found,
  vs_null_reference,
};

typedef struct VideoStream VideoStream;
typedef enum VideoStreamResult VideoStreamResult;


VideoStreamResult vs_open_format_context_from_path(char *path, AVFormatContext **fmt_ctx, int *err);

VideoStreamResult vs_open_codec_context(AVFormatContext *fmt_ctx, int stream_idx, uint32_t nThreads, int resolution, AVCodecContext **codec_ctx, int *err);

VideoStreamResult vs_create_sws_context(AVCodecContext *codec_ctx, struct SwsContext **sws_ctx,
  int new_width, int new_height, enum AVPixelFormat new_pix_fmt, int flags, const double *param, int *err);

VideoStreamResult vs_create_pkt_frm(AVPacket **pkt, AVFrame **frm, AVFrame **swsfrm);

void vs_free(VideoStream *vstream);


/// @brief Seek position in stream by combining a fast mode (seek keyframes) in compressed stream
///        and precise mode (decode up to right frame) in raw stream.
/// @param fmt_ctx AVFormatContext
/// @param stream AVStream
/// @param timestamp Seconds since start of stream in units of stream.time_base. 
/// @param flags AVSEEK_FLAG_BACKWARD, AVSEEK_FLAG_BYTE, AVSEEK_FLAG_ANY, AVSEEK_FLAG_FRAME. If flags < 0, then don't use fast-mode
/// @param codec_ctx_if_decode_frames Either codec_ctx or NULL if stream should not be decoded. So if NULL is provided, then don't use precise-mode. Automatically appends AVSEEK_FLAG_BACKWARD if not-NULL and fast-mode is used.
/// @param pkt Ignored if codec_ctx_if_decode_frames is NULL
/// @param frm Ignored if codec_ctx_if_decode_frames is NULL
/// @return VideoStreamResult
VideoStreamResult vs_seek(AVFormatContext *fmt_ctx, AVStream *stream, int64_t timestamp, int flags, AVCodecContext *codec_ctx_if_decode_frames, AVPacket *pkt, AVFrame *frm, int* err);

/// @brief See vs_seek
/// @param seconds Timestamp as double
VideoStreamResult vs_seek_at(AVFormatContext *fmt_ctx, AVStream *stream, double seconds, int flags, AVCodecContext *codec_ctx_if_decode_frames, AVPacket *pkt, AVFrame *frm, int* err);


/// @brief Decode nFrames number of new frames and optionally apply sws_context to last frame. If pkt->dts != frm->pkt_dts or frm->pkt_dts == AV_NOPTS_VALUE,
///        then it is assumed that frm is not up to date. If nFrames > 0, then it doesn't matter, otherwise the function sets frm to
///        the first frame that can be successfully decoded. It either is the current frame associated with pkt, or the next frame.
///        If swsfrm->pts == frm->pts, then it is assumed that swsfrm is already the decoded frm. So in order to force recalculating
///        swsfrm, set swsfrm->pts to AV_NOPTS_VALUE.
/// @param fmt_ctx AVStream
/// @param codec_ctx AVCodecContext
/// @param stream AVStream
/// @param pkt AVPacket
/// @param frm AVFrame
/// @param sws_ctx_if_scale Either sws_ctx or NULL if no scaling or pixel format conversion should be applied. Is only applied to last frame.
/// @param swsfrm Ignored if sws_ctx_if_scale is NULL. Otherwise result of sws_scale is stored here. Timestamps are copied from frm
/// @param nFrames Number of successful frames to decode
/// @return VideoStreamResult
VideoStreamResult vs_decode_frames(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm, struct SwsContext *sws_ctx_if_scale, AVFrame *swsfrm, uint64_t nFrames, int *err);

/// @brief See vs_decode_frames for nFrames = 0
VideoStreamResult vs_decode_current_frame(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm, struct SwsContext *sws_ctx_if_scale, AVFrame *swsfrm, int *err);

/// @brief See vs_decode_frames for nFrames = 1
VideoStreamResult vs_decode_next_frame(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm, struct SwsContext *sws_ctx_if_scale, AVFrame *swsfrm, int *err);

VideoStreamResult vs_decode(AVFormatContext *fmt_ctx, AVCodecContext *codec_ctx, AVStream *stream, AVPacket *pkt, AVFrame *frm, struct SwsContext *sws_ctx, AVFrame *swsfrm, const DecodingDecider *const decider, const DecodingActor *const actor, int *err);



#endif

