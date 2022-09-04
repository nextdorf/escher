#pragma once
#include<gtk/gtk.h>
//#include "GResource/escher.gresource.h"
#include "../EscherApp.h"



#define ESCHER_MAIN_WINDOW_TYPE (escher_main_window_get_type ())
G_DECLARE_FINAL_TYPE (EscherMainWindow, escher_main_window, ESCHER, MAIN_WINDOW, GtkApplicationWindow)


EscherMainWindow *escher_main_window_new (EscherApp *app);

void escher_main_window_open(EscherMainWindow *win, GFile *file);

//void activateMainWindow(GtkApplication *app, gpointer user_data);



