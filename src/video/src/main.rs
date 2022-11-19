use std::{fs::File, io::Write, path};

use video::{VideoStream, PartialVideoStream, SWS_Scaling, AVPixelFormat, Seek, VideoStreamErr};


fn main() -> Result<(), VideoStreamErr>{
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 3 || args.len() > 4{
    println!("Usage: {} video_path out_path [skip = 2m]", args[0]);
    Ok(())
  } else {
    let video_path = args[1].as_str();
    let out_path = args[2].as_str();
    let skip = parse_time_to_secs(args.get(3).unwrap_or(&"2m".to_string()));
    let stream_idx = 0;

    let video_path = path::Path::new(video_path);
    let vs: VideoStream = Ok(PartialVideoStream::new()).and_then(|pvs|
      pvs.open_format_context_from_path(video_path)?
        .open_codec_context(stream_idx, 0, -1)?
        .create_sws_context(1280, 720, AVPixelFormat::AV_PIX_FMT_RGB24, SWS_Scaling::Spline)?
        .create_pkt_frm()?
        .fmapMut(|vs| {
          vs.seek(skip, Seek::empty())?;
          vs.decode_frames(0, true)?;
          Ok(())
        })?
        .try_into()
      ).expect("Videostream could not be created with valid data");
    let raw_img = vs.decoded_frm();

    let mut f = match File::create(out_path) {
      Ok(g) => g,
      Err(_) => return Err(VideoStreamErr::IO)
    };
    for plane in raw_img.planes() {
      match f.write_all(plane) {
        Ok(()) => (),
        Err(_) => return Err(VideoStreamErr::IO)
      }
    }
    f.sync_data().or(Err(VideoStreamErr::IO))
  }
}


fn parse_time_to_secs(time: &str) -> f64 {
  let time_v: Vec<char> = time.chars().collect();
  let mut ret: f64 = 0.;
  let (mut i0, mut i1) = (0, 0);
    
  while i1 < time.len() {
    let c: char = time_v[i1].into();
    match c {
      'h'|'H'|'m'|'M'|'s'|'S' => {
        let insecs = match c {
          'h'|'H' => 60.*60.,
          'm'|'M' => 60.,
          's'|'S' => 1.,
          _ => panic!(),
        };
        ret += insecs*time[i0..i1].parse::<f64>().expect(format!("Could not parse {}", time).as_str());
        i0 = i1+1;
      },
      _ => ()
    };
    i1+=1;
  }
  if i0 < i1 {
    ret += time[i0..i1].parse::<f64>().expect(format!("Could not parse {}", time).as_str());
  }
  ret
}

