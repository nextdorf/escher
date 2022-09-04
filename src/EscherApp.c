#include "EscherApp.h"
#include "GUI/MainWindow.h"
#include "GUI/GResource/escher.gresource.h"

struct _EscherApp
{
  GtkApplication parent;
};

G_DEFINE_TYPE(EscherApp, escher_app, GTK_TYPE_APPLICATION);

static void escher_app_init (EscherApp *app){
}

static void escher_app_activate (GApplication *app) {
  EscherMainWindow *win;

  win = escher_main_window_new (ESCHER_APP (app));
  gtk_window_present (GTK_WINDOW (win));
}

static void escher_app_open (GApplication *app, GFile **files, int n_files, const char *hint){
  GList *windows;
  EscherMainWindow *win;
  int i;

  windows = gtk_application_get_windows (GTK_APPLICATION (app));
  if (windows)
    win = ESCHER_MAIN_WINDOW (windows->data);
  else
    win = escher_main_window_new (ESCHER_APP (app));

  for (i = 0; i < n_files; i++)
    escher_main_window_open (win, files[i]);

  gtk_window_present (GTK_WINDOW (win));
}

static void escher_app_class_init (EscherAppClass *class){
  g_resources_register(escher_get_resource());
  G_APPLICATION_CLASS (class)->activate = escher_app_activate;
  G_APPLICATION_CLASS (class)->open = escher_app_open;
}

EscherApp *escher_app_new (void){
  return g_object_new (ESCHER_APP_TYPE, "application-id", "com.github.nextdorf.escher",
    "flags", G_APPLICATION_HANDLES_OPEN, NULL);
}
