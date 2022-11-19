#include "VideoStream.h"
#include <stdio.h>
#include <stdint.h>

int my_renderframe(char *path, double seconds, int *width, int *height, int *outwidth, int *outheight, uint8_t **rgb);

double parseTimeInSecs(const char *s);
char *parseTimeFromSecs(double t);

void dumbFctxInfo(AVFormatContext *fctx, int defaultIdx);
void dumbCctxInfo(AVCodecContext *cctx);

size_t writeBMPHeader(FILE *f, uint32_t width, uint32_t height);


int main(int argc, char** argv){
  const double skip_secs_default = 10;

  int width = -1, height = -1, outwidth = 1280, outheight = 720;
  char *path;
  uint8_t *data = NULL;
  double skip_secs = 0;

  if(argc <= 1 || strcmp(argv[1], "--help") == 0){
    printf("Usage: %s path-to-video-file [skip = %fs]\n", argv[0], skip_secs_default);
    return 0;
  }
  if(argc >= 2)
    path = argv[1];
  if(argc >= 3)
    for(int i=2; i<argc; ++i)
      skip_secs += parseTimeInSecs(argv[i]);
  else
    skip_secs = skip_secs_default;

  char *skip_secs_str = parseTimeFromSecs(skip_secs);
  printf("Input:\n  Path:\t %s\n  skip:\t %s\n", path, skip_secs_str);
  free(skip_secs_str);

  int res = my_renderframe(path, skip_secs, &width, &height, &outwidth, &outheight, &data);

  printf("Result: %i\n  decoded size:\t %ix%i\n  final size:\t %ix%i\n", res, width, height, outwidth, outheight);


  FILE *f = fopen("raw.rgb", "wb");
  size_t nWrote = 0;
  // nWrote += writeBMPHeader(f, outwidth, outheight);
  nWrote += fwrite(data, 3, outwidth*outheight, f);
  fclose(f);
  free(data);

  return res;
}

int my_renderframe(char *path, double seconds, int *width, int *height, int *outwidth, int *outheight, uint8_t **rgb){
  AVFormatContext *fctx = NULL;
  AVCodecContext *cctx = NULL;
  struct SwsContext *swsctx = NULL;
  int err;

  vs_open_format_context_from_path(path, &fctx, &err);
  int idx = av_find_default_stream_index(fctx);
  dumbFctxInfo(fctx, idx);

  vs_open_codec_context(fctx, idx, 0, -1, &cctx, &err);
  // // cctx->lowres = 2;
  // cctx->thread_count = 4;
  // cctx->thread_type = FF_THREAD_FRAME;
  dumbCctxInfo(cctx);

  *width = cctx->width;
  *height = cctx->height;
  if(*outwidth < 0) *outwidth = *width;
  if(*outheight < 0) *outheight = *height;
  vs_create_sws_context(cctx, &swsctx, *outwidth, *outheight, AV_PIX_FMT_RGB24, SWS_SPLINE, NULL, &err);


  VideoStream *vstream = malloc(sizeof(VideoStream));
  vstream->fmt_ctx = fctx;
  vstream->codec_ctx = cctx;
  vstream->stream = fctx->streams[idx];
  // vstream->pkt = av_packet_alloc();
  // vstream->frm = av_frame_alloc();
  // vstream->swsfrm = av_frame_alloc();
  vs_create_pkt_frm(&(vstream->pkt), &(vstream->frm), &(vstream->swsfrm));
  vstream->sws_ctx = swsctx;


  vs_seek_at(vstream->fmt_ctx, vstream->stream, seconds, 0, vstream->codec_ctx, vstream->pkt, vstream->frm, &err);
  vs_decode_current_frame(vstream->fmt_ctx, vstream->codec_ctx, vstream->stream, vstream->pkt, vstream->frm, vstream->sws_ctx, vstream->swsfrm, &err);
  const size_t datasize = vstream->swsfrm->height * vstream->swsfrm->linesize[0];
  *rgb = malloc(datasize);
  memcpy(*rgb, vstream->swsfrm->data[0], datasize);


  vs_free(vstream);
  free(vstream);

  return err;
}

double parseTimeInSecs(const char *s){
  if(!s) return NAN;
  const size_t len = strlen(s);
  if(!len) return 0;

  char *tmp;
  double res;
  for(int i=0; i< len; i++){
    switch (s[i]) {
      case 's':
      case 'S':
        tmp = malloc(i+1);
        strncpy(tmp, s, i);
        tmp[i] = 0;
        res = atof(tmp);
        res += parseTimeInSecs(s + (i+1));
        free(tmp);
        return res;
      case 'm':
      case 'M':
        tmp = malloc(i+1);
        strncpy(tmp, s, i);
        tmp[i] = 0;
        res = atof(tmp)*60;
        res += parseTimeInSecs(s + (i+1));
        free(tmp);
        return res;
      case 'h':
      case 'H':
        tmp = malloc(i+1);
        strncpy(tmp, s, i);
        tmp[i] = 0;
        res = atof(tmp)*60*60;
        res += parseTimeInSecs(s + (i+1));
        free(tmp);
        return res;
      default:
        break;
    }
  }
  return atof(s);
}

char *parseTimeFromSecs(double t){
  const char invalidStr[] = "???";
  char *ret = malloc(1024);
  if(t < 0){
    strcpy(ret, invalidStr);
    return ret;
  }
  uint64_t tInt = t;
  uint64_t h = tInt/(60*60), m = (tInt % (60*60))/(60);
  double s = (tInt % 60) + (t - tInt);
  const char fmtFull[] = "%ih%02im%02.2fs";
  if(h == 0){
    if(m == 0)
      sprintf(ret, fmtFull + 3 + 5, s);
    else
      sprintf(ret, fmtFull + 3, m, s);
  }
  else
    sprintf(ret, fmtFull, h, m, s);
  return ret;
}

void dumbFctxInfo(AVFormatContext *fctx, int defaultIdx){
  printf("AVFormatContext:\n");

  printf("  Streams (len = %i)\n", fctx->nb_streams);
  for(int i=0; i<fctx->nb_streams; i++){
    AVStream *s = fctx->streams[i];
    double timebase = ((double)s->time_base.num)/s->time_base.den;
    double dur = timebase * s->duration;
    char *durStr = parseTimeFromSecs(dur);
    double startTime = timebase * s->start_time;
    int streamID = s->id;
    int64_t nFrames = s->nb_frames ? s->nb_frames : (s->duration ? -1 : 0);
    printf("  %s%i:\t nFrames = %i, dur = %s\n", (i==defaultIdx) ? "->":"  ", i, nFrames, durStr);
    free(durStr);
    AVDictionaryEntry *m = av_dict_get(s->metadata, "", NULL, AV_DICT_IGNORE_SUFFIX);
    if(m) {
      printf("      \t metadata = { %s: %s", m->key, m->value);
      while(m = av_dict_get(s->metadata, "", m, AV_DICT_IGNORE_SUFFIX))
        printf(", %s: %s", m->key, m->value);
      printf(" }\n");
    }
  }

  printf("  Programs (len = %i)\n", fctx->nb_programs);
  printf("  Chapters (len = %i)\n", fctx->nb_chapters);
  for(int i=0; i<fctx->nb_chapters; i++){
    AVChapter *ch = fctx->chapters[i];
    double timebase = ((double)ch->time_base.num)/ch->time_base.den;
    char *t1 = parseTimeFromSecs(timebase * ch->start), *t2 = parseTimeFromSecs(timebase * ch->end);
    int chapterID = ch->id;
    printf("    %i:\t dur = %s - %s\n", i, t1, t2);
    AVDictionaryEntry *m = av_dict_get(ch->metadata, "", NULL, AV_DICT_IGNORE_SUFFIX);
    if(m) {
      printf("      \t metadata = { %s: %s", m->key, m->value);
      while(m = av_dict_get(ch->metadata, "", m, AV_DICT_IGNORE_SUFFIX))
        printf(", %s: %s", m->key, m->value);
      printf(" }\n");
    }
  }
}

void dumbCctxInfo(AVCodecContext *cctx){
  printf("AVCodecContext:\n");

  const AVCodec *codec = cctx->codec;

  printf("  Name:\t %s (%s)\n", codec->name, codec->long_name);

  // printf("  Resolution:\t %s\n", cctx->lowres == 1 ? "1/2" : (cctx->lowres == 2 ? "1/4" : "Full"));
  if(cctx->lowres)
    printf("  Resolution:\t 1/%i (Lowest: 1/%i)\n", 1 << cctx->lowres, 1 << codec->max_lowres);
  else if(codec->max_lowres)
    printf("  Resolution:\t Full (Lowest: 1/%i)\n", 1 << codec->max_lowres);
  else
    printf("  Resolution:\t Full (Lower Resolutions not supported)\n");
  printf("  Threads:\t %i (%s)\n", cctx->thread_count,
    cctx->active_thread_type == FF_THREAD_FRAME ? "decode different frames in parallel" : 
    (cctx->active_thread_type == FF_THREAD_SLICE ? "decode several slices per frame in parallel" :
    "unknown parallelization mode"));

  size_t ncaps = 0;
  const char *const capabilityNames[] = {
    "DRAW_HORIZ_BAND",            // 1<<0
    "DR1",                        // 1<<1
    NULL,
    "TRUNCATED",                  // 1<<3
    NULL,
    "DELAY",                      // 1<<5
    "SMALL_LAST_FRAME",           // 1<<6
    NULL,
    "SUBFRAMES",                  // 1<<8
    "EXPERIMENTAL",               // 1<<9
    "CHANNEL_CONF",               // 1<<10
    NULL,
    "FRAME_THREADS",              // 1<<12
    "SLICE_THREADS",              // 1<<13
    "PARAM_CHANGE",               // 1<<14
    "OTHER_THREADS/AUTO_THREADS", // 1<<15
    "VARIABLE_FRAME_SIZE",        // 1<<16
    "AVOID_PROBING",              // 1<<17
    "HARDWARE",                   // 1<<18
    "HYBRID",                     // 1<<19
    "ENCODER_REORDERED_OPAQUE",   // 1<<20
    "ENCODER_FLUSH",              // 1<<21
    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 
    "INTRA_ONLY",                 // 1<<30
    "LOSSLESS",                   // 1<<31
  };
  int8_t caps[sizeof(capabilityNames)/sizeof(*capabilityNames)] = {-1};
  for(int i=0; i<sizeof(caps)/sizeof(*caps); i++)
    if(codec->capabilities & (1 << i))
      caps[ncaps++] = i;
  if(ncaps){
    printf("  Capabilities:\t %i", caps[0]);
    for(int i=1; i<ncaps; ++i)
      printf(", %i", caps[i]);
    printf("\n    %s", capabilityNames[caps[0]]);
    for(int i=1; i<ncaps; ++i)
      printf(", %s", capabilityNames[caps[i]]);
    printf("\n");
  } else {
    printf("  Capabilities: None\n");
  }

  // printf("  b-frames:\t up to %i\n", cctx->max_b_frames);
}


size_t writeBMPHeader(FILE *f, uint32_t width, uint32_t height){
  uint8_t header[14 + 48];
  memcpy(header, "BM", 2);
  *((uint32_t *)(header+2)) = sizeof(header) + 3*width*height;
  memcpy(header+6, "yolo", 4);
  *((uint32_t *)(header+10)) = sizeof(header);

  *((uint32_t *)(header+14)) = sizeof(header)-14;
  *((int64_t *)(header+18)) = width;
  *((int64_t *)(header+26)) = height;
  *((uint16_t *)(header+34)) = 1;
  *((uint16_t *)(header+36)) = 24; //RGB24
  *((uint32_t *)(header+38)) = 0; //compression
  *((uint32_t *)(header+42)) = 3*width*height;
  *((uint32_t *)(header+46)) = 2835; //DPI
  *((uint32_t *)(header+50)) = 2835; //DPI
  *((uint32_t *)(header+54)) = 0;
  *((uint32_t *)(header+58)) = 0;

  return fwrite(header, 1, sizeof(header), f);
}


