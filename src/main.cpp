#include<iostream>
#include<gtk/gtk.h>

extern "C" {
  #include "GUI/MainWindow.h"
}


int main(int argc, char* argv[]){
	std::cout << "Hello, Twitch!" << std::endl;

  g_resources_register(escher_get_resource());
	GtkApplication *app;

  int status;

  app = gtk_application_new ("com.github.nextdorf.escher", G_APPLICATION_FLAGS_NONE);
  g_signal_connect (app, "activate", G_CALLBACK (activateMainWindow), NULL);
  status = g_application_run (G_APPLICATION (app), argc, argv);
  g_object_unref (app);

  return status;
}

