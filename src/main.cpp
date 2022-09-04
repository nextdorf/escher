#include<iostream>
#include<gtk/gtk.h>

extern "C" {
  #include "EscherApp.h"
}


int main(int argc, char* argv[]){
	std::cout << "Hello, Twitch!" << std::endl;

  int status;
  status = g_application_run (G_APPLICATION (escher_app_new ()), argc, argv);

  return status;
}

