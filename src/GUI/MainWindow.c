#include "MainWindow.h"

struct _EscherMainWindow
{
  GtkApplicationWindow parent;
  GtkWidget *stack;
  GtkButton *button1;
};

G_DEFINE_TYPE(EscherMainWindow, escher_main_window, GTK_TYPE_APPLICATION_WINDOW);


static void print_hello (GtkWidget *widget, gpointer data){
  g_print ("Hello World\n");
}


static void escher_main_window_init (EscherMainWindow *win){
  gtk_widget_init_template (GTK_WIDGET (win));

  g_signal_connect (win->button1, "clicked", G_CALLBACK (print_hello), NULL);
}

static void escher_main_window_class_init (EscherMainWindowClass *class){
  gtk_widget_class_set_template_from_resource (GTK_WIDGET_CLASS (class), "/gui/MainWindow.ui");
  gtk_widget_class_bind_template_child (GTK_WIDGET_CLASS (class), EscherMainWindow, stack);
  gtk_widget_class_bind_template_child (GTK_WIDGET_CLASS (class), EscherMainWindow, button1);

}

EscherMainWindow *escher_main_window_new (EscherApp *app){
  return g_object_new (ESCHER_MAIN_WINDOW_TYPE, "application", app, NULL);
}

void escher_main_window_open(EscherMainWindow *win, GFile *file){
  g_print("Opening File...\n");

  char *basename;
  GtkWidget *scrolled, *view;
  char *contents;
  gsize length;

  basename = g_file_get_basename (file);

  scrolled = gtk_scrolled_window_new ();
  gtk_widget_set_hexpand (scrolled, TRUE);
  gtk_widget_set_vexpand (scrolled, TRUE);
  view = gtk_text_view_new ();
  gtk_text_view_set_editable (GTK_TEXT_VIEW (view), FALSE);
  gtk_text_view_set_cursor_visible (GTK_TEXT_VIEW (view), FALSE);
  gtk_scrolled_window_set_child (GTK_SCROLLED_WINDOW (scrolled), view);
  gtk_stack_add_titled (GTK_STACK (win->stack), scrolled, basename, basename);

  if (g_file_load_contents (file, NULL, &contents, &length, NULL, NULL))
    {
      GtkTextBuffer *buffer;

      buffer = gtk_text_view_get_buffer (GTK_TEXT_VIEW (view));
      gtk_text_buffer_set_text (buffer, contents, length);
      g_free (contents);
    }

  g_free (basename);

}

///////////////

// static void print_hello (GtkWidget *widget, gpointer data){
//   g_print ("Hello World\n");
// }

// static void quit_cb (GtkWindow *window){
//   gtk_window_close (window);
// }


// void activateMainWindow (GtkApplication *app, gpointer user_data){
//   /* Construct a GtkBuilder instance and load our UI description */
//   GtkBuilder *builder = gtk_builder_new ();
//   //gtk_builder_add_from_file (builder, "example.ui", NULL);

//   //gtk_builder_add_from_resource (builder, "/gui/GUI/GResource/MainWindow.ui", NULL);
//   gtk_builder_add_from_resource (builder, "/gui/MainWindow.ui", NULL);
  

//   /* Connect signal handlers to the constructed widgets. */
//   GObject *window = gtk_builder_get_object (builder, "window");
//   gtk_window_set_application (GTK_WINDOW (window), app);

//   GObject *button = gtk_builder_get_object (builder, "button1");
//   g_signal_connect (button, "clicked", G_CALLBACK (print_hello), NULL);

//   button = gtk_builder_get_object (builder, "button2");
//   g_signal_connect (button, "clicked", G_CALLBACK (print_hello), NULL);

//   button = gtk_builder_get_object (builder, "quit");
//   g_signal_connect_swapped (button, "clicked", G_CALLBACK (quit_cb), window);

//   gtk_widget_show (GTK_WIDGET (window));

//   /* We do not need the builder any more */
//   g_object_unref (builder);
// }



