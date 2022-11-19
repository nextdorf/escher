extern crate cmake;
extern crate pkg_config;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn invoke_cmake(){
  // Builds the project in the directory located in `libfoo`, installing it
  // into $OUT_DIR
  let dst = cmake::build("libvideoc");

  println!("cargo:rustc-link-search=native={}", dst.display());
  println!("cargo:rustc-link-lib=static=videoc");
}

fn invoke_buildgen(){
  // Write the bindings to the $OUT_DIR/bindings.rs file.
  let out_path = env::var("OUT_DIR").unwrap();
  let cmake_install_datarootdir = format!("{out_path}/share"); //TODO: get this from cmake crate

  /*
  // Tell cargo to look for shared libraries in the specified directory
  // println!("cargo:rustc-link-search=/path/to/lib");
  println!("cargo:rustc-link-search=libvideoc/install/libs");

  // Tell cargo to tell rustc to link the system bzip2
  // shared library.
  println!("cargo:rustc-link-lib=videoc");
  */

  // Tell cargo to invalidate the built crate whenever the wrapper changes
  println!("cargo:rerun-if-changed=wrapper_videoc.h");
  println!("cargo:rerun-if-changed=libvideoc/src");
  println!("cargo:rerun-if-changed=libvideoc/include");
  println!("cargo:rerun-if-changed=libvideoc/CMakeLists.txt");
  println!("cargo:rerun-if-changed=libvideoc/videoc.pc.in");

  std::env::set_var("PKG_CONFIG_PATH", format!("{}/pkgconfig:$PKG_CONFIG_PATH", cmake_install_datarootdir));
  // eprintln!("{}", std::env::var("PKG_CONFIG_PATH").unwrap());
  let _pkgconf = pkg_config::Config::new()
    .probe("videoc")
    .expect("Unable to config from videoc.pc");

  // The bindgen::Builder is the main entry point
  // to bindgen, and lets you build up options for
  // the resulting bindings.
  let bindings = bindgen::Builder::default()
    // .clang_args(iter)
    // The input header we would like to generate
    // bindings for.
    .header("wrapper_videoc.h")
    // .header("libvideoc/install/include/renderframe.h")
    //Only public interface to library
    .allowlist_function("vs_.*")
    .allowlist_type("VideoStream.*")
    .rustified_enum("VideoStreamResult")
    .rustified_enum("DecodingDecision")
    .rustified_enum("DecodingDecisionIdx")
    .rustified_enum("DecodingAction")
    .rustified_enum("DecodingActionIdx")

    .rustified_enum("AVSampleFormat")
    .rustified_enum("AVFrameSideDataType")
    .rustified_enum("AVMediaType")
    .rustified_enum("AVPictureType")
    .rustified_enum("AVClassCategory")
    .rustified_enum("AVPixelFormat")
    .rustified_enum("AVColorPrimaries")
    .rustified_enum("AVColorTransferCharacteristic")
    .rustified_enum("AVColorSpace")
    .rustified_enum("AVColorRange")
    .rustified_enum("AVChromaLocation")
    .rustified_enum("AVChannel")
    .rustified_enum("AVCodecID")
    .rustified_enum("AVFieldOrder")
    .rustified_enum("AVAudioServiceType")
    .rustified_enum("AVPacketSideDataType")
    .rustified_enum("AVDurationEstimationMethod")
    
    .allowlist_var("SWS_FAST_BILINEAR")
    .allowlist_var("SWS_BILINEAR")
    .allowlist_var("SWS_BICUBIC")
    .allowlist_var("SWS_X")
    .allowlist_var("SWS_POINT")
    .allowlist_var("SWS_AREA")
    .allowlist_var("SWS_BICUBLIN")
    .allowlist_var("SWS_GAUSS")
    .allowlist_var("SWS_SINC")
    .allowlist_var("SWS_LANCZOS")
    .allowlist_var("SWS_SPLINE")

    .allowlist_var("AVSEEK_FLAG_BACKWARD")
    .allowlist_var("AVSEEK_FLAG_BYTE")
    .allowlist_var("AVSEEK_FLAG_ANY")
    .allowlist_var("AVSEEK_FLAG_FRAME")
    // .allowlist_recursively(false)
    // .allowlist_file(out_path.join("install/include/videoc.h").to_str().unwrap())
    // .allowlist_file(out_path.join("install/include/renderframe.h").to_str().unwrap())
    // Tell cargo to invalidate the built crate whenever any of the
    // included header files changed.
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

  bindings
    .write_to_file(PathBuf::from(out_path).join("bindings.rs"))
    .expect("Couldn't write bindings!");
}

fn main() {
  invoke_cmake();
  invoke_buildgen();
}
