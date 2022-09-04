#pragma once
#include <gtk/gtk.h>


#define ESCHER_APP_TYPE (escher_app_get_type ())
G_DECLARE_FINAL_TYPE (EscherApp, escher_app, ESCHER, APP, GtkApplication)

EscherApp *escher_app_new (void);
